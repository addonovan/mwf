extern crate iron;

use std::collections::HashMap;
use view::ViewResult;
use handle::RequestHandler;

/// Maps a route's variables to their respective values
pub type RouteMap = HashMap<String, String>;

/// Resolves a route by using the path specification supplied to it.
pub trait Resolver
    where Self: Send + Sync
{
    /// Resolves a request `route` to returns a `RouteMap` binding
    /// any route-specific information to a key.
    fn resolve(&self, route: &Vec<&str>) -> Option<RouteMap>;
}

/// The constructor for a [Resolver].
type ResolverConstructor = Fn(Vec<String>) -> Box<Resolver>;

/// A resolver which follows the standard resolving method
/// described by [StandardRouter].
///
/// The standard resolver has three possible URL tokens:
/// * `Literal`: Matches the text given exactly. This is the default
/// * `Variable`: Matches any text and stores the actual value in the [RouteMap]
///   with the variable's name. These are denoted with a leading `:`, which is
///   included in the variable's name in the [RouteMap].
/// * `Optional`: Matches any text (or none at all) and saves the value in the
///   RouteMap, like the variable matcher. This is denoted with a leading `:`
///   and a trailing `?`, both of which are used in the variable's name.
///
/// Some examples of route specifications for a standard resolver, and their
/// corresponding `RouteMap`:
///
/// Specification: `/foo`
/// Route           | Matches | Route Map Entries
/// --------------- | ------- | -----------------
/// `/foo`          | Yes     | `{}`
/// `/foo/bar`      | No      |
/// `/foo/bar/baz`  | No      |
/// `/foo/bar/qux`  | No      |
/// `/foo/baz`      | No      |
///
/// Specification `/foo/:bar`
/// Route           | Matches | Route Map Entries
/// --------------- | ------- | -----------------
/// `/foo`          | No      |
/// `/foo/bar`      | Yes     | `{":bar": "bar"}`
/// `/foo/bar/baz`  | No      |
/// `/foo/bar/qux`  | No      |
/// `/foo/baz`      | Yes     | `{":bar": "baz"}`
///
/// Specification `/foo/:bar?`
/// Route           | Matches | Route Map Entries
/// --------------- | ------- | -----------------
/// `/foo`          | Yes     | `{":bar": ""}`
/// `/foo/bar`      | Yes     | `{":bar": "bar"}`
/// `/foo/bar/baz`  | No      |
/// `/foo/bar/qux`  | No      |
/// `/foo/baz`      | Yes     | `{":bar": "baz"}`
///
/// Specification `/foo/:bar/:baz?`
/// Route           | Matches | Route Map Entries
/// --------------- | ------- | -----------------
/// `/foo`          | No      |
/// `/foo/bar`      | Yes     | `{":bar": "bar", ":baz?": ""}`
/// `/foo/bar/baz`  | Yes     | `{":bar": "bar", ":baz?": "baz"}`
/// `/foo/bar/qux`  | Yes     | `{":bar": "bar", ":baz?": "quz"}`
/// `/foo/baz`      | Yes     | `{":bar": "baz"}`
///
pub struct StandardResolver
{
    /// The routing specification
    route: Vec<RouteSpec>
}

/// A more user-friendly interface to build a [Router] from.
///
/// By default, this will use the [StandardResolver] for paths, but this can
/// be changed.
pub struct RouterBuilder
{
    /// The constructor used to create new resolvers.
    constructor: Box<ResolverConstructor>,

    /// The list of resolvers which have been bound to actions.
    resolvers: Vec<(Box<Resolver>, Box<RequestHandler>)>,
}

/// The router which the server will use while running to handle requests.
/// This is a finalized version of the [RouterBuilder] which is completely
/// thread-safe. However, once the router has been constructed, the resolvers
/// are static and cannot change.
pub struct Router
{
    /// The list of resolvers which could possible handle a request and the
    /// handlers associated with them.
    resolvers: Vec<(Box<Resolver>, Box<RequestHandler>)>,
}

/// Describes the parts of a route specification in a [StandardResolver].
enum RouteSpec
{
    /// A required string literal. The interior value is the string to match.
    Literal(String),

    /// A required variable. The interior value is the name of the variable.
    Variable(String),

    /// An optional variable. The interior value is the name of the variable.
    Optional(String),
}

//
// Implementation
//

impl RouterBuilder
{
    /// Builds a new router builder, which can be used in places like a
    /// [ServerBuilder] to construct the [Resolver]s which will be used
    /// to handle a request.
    pub fn new() -> RouterBuilder
    {
        RouterBuilder {
            constructor: Box::new(StandardResolver::new),
            resolvers: Vec::new(),
        }
    }

    /// Changes the `constructor` to use when creating new resolvers.
    pub fn constructor(&mut self, constructor: Box<ResolverConstructor>)
    {
        self.constructor = constructor;
    }

    /// Binds a URL route to an action in the receiver object.
    pub fn bind<T: Into<String>, H: 'static>(&mut self, path: T, handler: H)
        where H: RequestHandler
    {
        let path: String = path.into();

        // now we split on all of the slashes, remove empty strings and convert
        // to owned strings
        let path: Vec<String> = path.split("/")
            .map(String::from)
            .filter_map(|it| {
                if it.is_empty() {
                    None
                }
                else {
                    Some(it)
                }
            })
            .collect();

        // now, we need to instance a new resolver using our constructor
        // and add it to our list of resolvers
        let constructor = &self.constructor;
        let resolver = constructor(path);
        let handler = Box::new(handler);
        self.resolvers.push((resolver, handler));
    }
}

impl Into<Router> for RouterBuilder
{
    fn into(self) -> Router
    {
        Router {
            resolvers: self.resolvers
        }
    }
}

impl Router
{
    pub fn handle(&self, request: &iron::Request) -> Option<ViewResult>
    {
        // remove all of the empty strings.
        let route: Vec<&str> = request.url.path()
            .iter()
            .filter_map(|it| {
                if it.is_empty() {
                    None
                }
                    else {
                        Some(*it)
                    }
            })
            .collect();

        for entry in self.resolvers.iter() {
            let &(ref resolver, ref handler) = entry;

            // if we can successfully resolve this route, then
            // we can just return whatever the handler yields.
            let data = match resolver.resolve(&route) {
                None => continue,
                Some(x) => x,
            };

            return Some(handler.handle(data));
        }

        // we weren't able to find anything to handle the request
        None
    }
}

impl StandardResolver
{
    /// Constructs a new [StandardResolver] by using the given `route`
    /// specification.
    fn new(route: Vec<String>) -> Box<Resolver>
    {
        // build a vector of route types now, so we don't have to do
        // any difficult processing later :^)
        let mut specs = Vec::new();
        for spec in route {
            if spec.starts_with(":") {
                if spec.ends_with("?") {
                    specs.push(RouteSpec::Optional(spec));
                }
                else {
                    specs.push(RouteSpec::Variable(spec));
                }
            }
            else {
                specs.push(RouteSpec::Literal(spec));
            }
        }

        Box::new(StandardResolver {
            route: specs
        })
    }
}

impl Resolver for StandardResolver
{
    fn resolve(&self, route: &Vec<&str>) -> Option<RouteMap>
    {
        let mut map = RouteMap::new();
        let mut i = 0;
        let mut j = 0;

        while let Some(expected) = self.route.get(i) {
            let actual = route.get(j);

            match expected {
                // if a literal doesn't match, then the path is wrong
                &RouteSpec::Literal(ref expected) => {

                    // we require there to be something in the actual route
                    let actual = actual?;
                    if actual != &expected.as_str() {
                        return None;
                    }

                    // and we need to move to the next part of the given route
                    j += 1;
                },

                // variables have to match
                &RouteSpec::Variable(ref name) => {
                    // we need something here, it's not optional
                    let actual = actual?.to_string();
                    let name = name.to_string();

                    // if there was already an item with a name there, we'll
                    // have to panic
                    if let Some(_) = map.insert(name, actual) {
                        panic!("Multiple variables with same name!");
                    }

                    // move onto the next route token
                    j += 1;
                },

                &RouteSpec::Optional(ref name) => {
                    let text: String = match actual {
                        None => "".into(),
                        Some(x) => x.to_string(),
                    };
                    let name = name.to_string();

                    if let Some(_) = map.insert(name, text) {
                        panic!("Multiple variables with the same name!");
                    }

                    j += 1;
                },
            }

            i += 1;
        }

        // if we still have actual tokens left, then we can't have possibly
        // matched the route
        if j < route.len() {
            None
        }
        else {
            Some(map)
        }
    }
}

#[cfg(test)]
mod test
{
    use routing::*;

    macro_rules! resolver {
        ( $( $x:expr ),* ) => {{
            let mut route = Vec::new();
            $(
                route.push($x.to_owned());
            )*
            StandardResolver::new(route)
        }}
    }

    macro_rules! resolve {
        ( $r:ident, $( $x:expr ),* ) => {{
            let mut test = Vec::new();
            $(
                test.push($x);
            )*
            $r.resolve(&test)
        }}
    }

    #[test]
    fn standard_matches_literals()
    {
        // Resolver: /
        let resolver = resolver![""];
        match resolve!(resolver, "") {
            None => panic!("/ didn't match /"),
            Some(map) => assert_eq!(0, map.len())
        }

        match resolve!(resolver, "foo") {
            Some(_) => panic!("/ matched /foo"),
            None => {}
        }

        match resolve!(resolver, "foo", "bar") {
            Some(_) => panic!("/ matched /foo/bar"),
            None => {}
        }

        // Resolver: /foo
        let resolver = resolver!["foo"];
        match resolve!(resolver, "") {
            Some(_) => panic!("/foo matched /"),
            None => {}
        }

        match resolve!(resolver, "foo") {
            None => panic!("/foo didn't match /foo"),
            Some(map) => assert_eq!(0, map.len())
        }

        match resolve!(resolver, "foo", "bar") {
            Some(_) => panic!("/foo matched /foo/bar"),
            None => {}
        }

        // Resolver: /foo/bar
        let resolver = resolver!["foo", "bar"];
        match resolve!(resolver, "") {
            Some(_) => panic!("/foo/bar matched /"),
            None => {},
        }

        match resolve!(resolver, "foo") {
            Some(_) => panic!("/foo/bar matched /foo"),
            None => {},
        }

        match resolve!(resolver, "foo", "bar") {
            None => panic!("/foo/bar didn't match /foo/bar"),
            Some(map) => assert_eq!(0, map.len())
        }
    }

    #[test]
    fn standard_matches_vars()
    {
        // Resolver: /:foo/:bar
        let resolver = resolver![":foo", ":bar"];

        match resolve!(resolver, "") {
            Some(_) => panic!("/:foo/:bar matched /"),
            None => {}
        }

        match resolve!(resolver, "foo") {
            Some(_) => panic!("/:foo/:bar matched /foo"),
            None => {}
        }

        match resolve!(resolver, "foo", "bar") {
            None => panic!("/:foo/:bar didn't match /foo/bar"),
            Some(map) => {
                assert_eq!(Some("foo"), map.get(":foo").map(String::as_str));
                assert_eq!(Some("bar"), map.get(":bar").map(String::as_str));
            }
        }

        match resolve!(resolver, "foo", "bar", "baz") {
            Some(_) => panic!("/:foo/:bar matched /foo/bar/baz"),
            None => {}
        }
    }

    #[test]
    fn standard_matches_optionals()
    {
        // Resolver: /:foo?/:bar?
        let resolver = resolver![":foo?", ":bar?"];

        match resolve!(resolver, "") {
            None => panic!("/:foo?/:bar? didn't match /"),
            Some(map) => {
                assert_eq!(Some(""), map.get(":foo?").map(String::as_str));
                assert_eq!(Some(""), map.get(":bar?").map(String::as_str));
            }
        }

        match resolve!(resolver, "foo") {
            None => panic!("/:foo?/:bar? didn't match /foo"),
            Some(map) => {
                assert_eq!(Some("foo"), map.get(":foo?").map(String::as_str));
                assert_eq!(Some(""), map.get(":bar?").map(String::as_str));
            }
        }

        match resolve!(resolver, "foo", "bar") {
            None => panic!("/:foo?/:bar? didn't match /foo/bar"),
            Some(map) => {
                assert_eq!(Some("foo"), map.get(":foo?").map(String::as_str));
                assert_eq!(Some("bar"), map.get(":bar?").map(String::as_str));
            }
        }

        match resolve!(resolver, "foo", "bar", "baz") {
            Some(_) => panic!("/:foo?/:bar? matched /foo/bar/baz"),
            None => {}
        }
    }
}
