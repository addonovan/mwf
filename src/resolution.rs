use std::collections::HashMap;

use hyper::Method;

/// A map of variables to their values in the route path.
///
/// Although, with non-standard resolvers, this could potentially
/// contain any information the resolver wishes to pass along to the
/// request handler.
pub type RouteMap = HashMap<String, String>;

/// A name tuple of the parameters given to a resolver.
pub struct ResolveParams<'a>
{
    /// The method of request (i.e. GET or POST)
    pub method: Method,

    /// The actual route requested from the server
    pub route: Vec<&'a str>,
}

/// Resolves a route and accepts it if it matched the parameters with which
/// it was constructed.
pub trait Resolver
    where Self: Send + Sync
{
    /// Attempts to resolve the given `params`. If it was successfully, it will
    /// return the filled [RouteMap], otherwise it will return `None`.
    fn resolve(&self, params: &ResolveParams) -> Option<RouteMap>;
}

//
// Standard Resolver
//

/// A token for the standard resolver.
enum Token
{
    /// A part of the route which has to match exactly. Its value is the content
    /// which must match.
    Literal(String),

    /// A part of the route which must be present, but will match anything. Its
    /// value is the name of the variable.
    Variable(String),

    /// A part of the route which might be present, and will match anything. Its
    /// value is the name of the optional variable.
    Optional(String),
}

/// The standard and default route resolver for mwf.
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
pub struct StandardResolver
{
    /// the request method (e.g. GET or POST)
    method: Method,

    /// The route specifiacation
    spec: Vec<Token>,
}

impl StandardResolver
{
    /// Creates a new standard resolver which requires the given connection
    /// `method` and follows the given route `spec`.
    pub fn new(method: Method, spec: Vec<String>) -> Box<Resolver>
    {
        let spec = spec.into_iter()
            .map(|token| {
                if token.starts_with(":") {
                    if token.ends_with("?") {
                        Token::Optional(token)
                    }
                    else {
                        Token::Variable(token)
                    }
                }
                else {
                    Token::Literal(token)
                }
            })
            .collect();

        Box::new(StandardResolver {
            method,
            spec,
        })
    }
}

impl Resolver for StandardResolver
{
    fn resolve(&self, params: &ResolveParams) -> Option<RouteMap>
    {
        // resolution MUST have the same request method
        if params.method != self.method {
            return None;
        }

        let mut map = RouteMap::new();
        let mut i = 0;

        while let Some(expected) = self.spec.get(i) {
            let actual = params.route.get(i);

            match expected {
                &Token::Literal(ref expected) => {
                    let actual = actual?;
                    if actual != &expected.as_str() {
                        return None;
                    }
                },

                &Token::Variable(ref name) => {
                    let actual = actual.map(|x| x.to_string())?;
                    let name = name.clone();

                    if let Some(_) = map.insert(name, actual) {
                        panic!("Multiple variables with the same name!");
                    }
                },

                &Token::Optional(ref name) => {
                    let text: String = match actual {
                        None => "".into(),
                        Some(x) => x.to_string(),
                    };
                    let name = name.clone();

                    if let Some(_) = map.insert(name, text) {
                        panic!("Multiple variables with the same name!");
                    }
                }
            }

            i += 1;
        }

        // if we still have more route to match, then we can't match
        if i < params.route.len() {
            None
        }
        else {
            Some(map)
        }
    }
}

