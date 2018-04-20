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

#[cfg(test)]
mod test
{
    use super::*;

    /// Creates a resolver that matches the given `method` and with the given
    /// path specs, `x`.
    macro_rules! resolver {
        ( $method:expr, $( $x:expr ),* ) => {{
            let mut route = Vec::new();
            $(
                route.push($x.to_owned());
            )*
            StandardResolver::new($method, route)
        }}
    }

    /// Tries to resolve the given path, `x`, using the resolver, `r`, and
    /// request `method`.
    macro_rules! resolve {
        ( $r:ident, $method:expr, $( $x:expr ),* ) => {{
            let mut test = Vec::new();
            $(
                test.push($x);
            )*
            let params = ResolveParams {
                method: $method,
                route: test,
            };
            $r.resolve(&params)
        }}
    }

    /// Tests if the root directory is matched via both GET and POST requests.
    #[test]
    fn standard_matches_root()
    {
        let resolver = resolver!(Method::Get, "");
        let map = resolve!(resolver, Method::Get, "")
            .expect("GET/ did not match GET/");
        assert_eq!(0, map.len());

        let resolver = resolver!(Method::Post, "");
        let map = resolve!(resolver, Method::Post, "")
            .expect("POST/ did not match POST/");
        assert_eq!(0, map.len());
    }

    /// Tests if the standard will reject routes based solely on the request
    /// method.
    #[test]
    fn standard_reject_wrong_method()
    {
        let resolver = resolver!(Method::Get, "");
        let map = resolve!(resolver, Method::Post, "");
        assert!(map.is_none());
    }

    /// Tests if the standard will match a series of path literals
    #[test]
    fn standard_matches_literals()
    {
        let resolver = resolver!(Method::Get, "foo", "bar", "baz");
        let map = resolve!(resolver, Method::Get, "foo", "bar", "baz")
            .expect("GET/foo/bar/baz did not match GET/foo/bar/baz");
        assert_eq!(0, map.len());
    }

    /// Tests if the standard will match a series of incorrect path literals.
    #[test]
    fn standard_rejects_wrong_literals()
    {
        let resolver = resolver!(Method::Get, "foo", "bar", "baz");
        let map = resolve!(resolver, Method::Get, "foo", "bar", "qux");
        assert!(map.is_none(), "GET/foo/bar/qux matched GET/foo/bar/baz");
    }

    /// Tests if the standard will match route variables and insert the text
    /// into the routemap.
    #[test]
    fn standard_matches_variable()
    {
        let resolver = resolver!(Method::Get, ":foo");
        let map = resolve!(resolver, Method::Get, "bar")
            .expect("GET/bar did not match GET/:foo");
        assert_eq!(1, map.len());
        assert_eq!(Some(&"bar".into()), map.get(":foo"));
    }

    /// Tests if the standard will match multiple route variables.
    #[test]
    fn standard_matches_multiple_variables()
    {
        let resolver = resolver!(Method::Get, ":foo", ":bar");
        let map = resolve!(resolver, Method::Get, "baz", "qux")
            .expect("GET/baz/qux did not match GET/:foo/:bar");
        assert_eq!(2, map.len());
        assert_eq!(Some(&"baz".into()), map.get(":foo"));
        assert_eq!(Some(&"qux".into()), map.get(":bar"));
    }

    /// Tests if the standard will reject if one of the variables is missing
    #[test]
    fn standard_rejects_missing_variables()
    {
        let resolver = resolver!(Method::Get, ":foo", ":bar");
        let map = resolve!(resolver, Method::Get, "baz");
        assert!(map.is_none(), "GET/baz matched GET/:foo/:bar");
    }

    /// Tests if the standard will match a missing optional variable and
    /// insert an empty string in its place in the routemap.
    #[test]
    fn standard_matches_missing_optional()
    {
        let resolver = resolver!(Method::Get, ":foo?");
        let map = resolve!(resolver, Method::Get, "")
            .expect("GET/ did not match GET/:foo?");
        assert_eq!(1, map.len());
        assert_eq!(Some(&"".into()), map.get(":foo?"));
    }

    /// Tests if the standard will match against a present optional variable
    /// and insert the value in its place in the routemap.
    #[test]
    fn standard_matches_present_optional()
    {
        let resolver = resolver!(Method::Get, ":foo?");
        let map = resolve!(resolver, Method::Get, "bar")
            .expect("GET/bar did not match GET/:foo?");
        assert_eq!(1, map.len());
        assert_eq!(Some(&"bar".into()), map.get(":foo?"));
    }

    /// Tests if the standard will correctly match a mix of literals, variables,
    /// and optional variables.
    #[test]
    fn standard_matches_mixed()
    {
        let resolver = resolver!(Method::Get, "foo", ":bar", ":baz?");
        let map = resolve!(resolver, Method::Get, "foo", "qux", "quux" )
            .expect("GET/foo/qux/quux did not match GET/foo/:bar/:baz?");
        assert_eq!(2, map.len());
        assert_eq!(Some(&"qux".into()), map.get(":bar"));
        assert_eq!(Some(&"quux".into()), map.get(":baz?"));

        let map = resolve!(resolver, Method::Get, "foo", "qux")
            .expect("GET/foo/qux did not match GET/foo/:bar/:baz?");
        assert_eq!(2, map.len());
        assert_eq!(Some(&"qux".into()), map.get(":bar"));
        assert_eq!(Some(&"".into()), map.get(":baz?"));
    }

    /// Tests if the standard will correctly reject routes which do not match
    /// the mixed specification.
    #[test]
    fn standard_rejects_mixed()
    {
        let resolver = resolver!(Method::Get, "foo", ":bar", ":baz?");
        let map = resolve!(resolver, Method::Get, "foo");
        assert!(map.is_none(), "GET/foo matched GET/foo/:bar/:baz?");

        let map = resolve!(resolver, Method::Get, "corge", "quux", "quuz");
        assert!(
            map.is_none(),
            "GET/corge/quux/quuz matched GET/foo/:bar/:baz?"
        );

        let map = resolve!(resolver, Method::Post, "qux", "quux", "quuz");
        assert!(
            map.is_none(),
            "POST/qux/quux/quuz matched GET/foo/:bar/:baz?"
        );
    }

}
