use std::collections::HashMap;

/// Maps a route's variables to their respective values
pub type RouteMap = HashMap<String, String>;

/// Creates instances of [RouteResolver]s, so the way pages are
/// routed can be easily changed.
///
/// See [StandardRouter] for an example of what `Routers` do.
pub trait Router
    where Self: Send + Sync
{
    /// Creates a new `Router` instance using the `route_spec` which is
    /// given by the user when registering a page callback.
    fn resolver(&self, route_spec: String) -> Box<RouteResolver>;
}

/// Resolves a route.
pub trait RouteResolver
    where Self: Send + Sync
{
    /// Resolves a request `route` to returns a `RouteMap` binding
    /// any route-specific information to a key.
    fn resolve(&self, route: &Vec<&str>) -> Option<RouteMap>;
}

/// The standard routing algorithm.
///
/// A route is matched literally, except for sections which begin
/// with a `:`, which are treated as variables. The actual text in
/// the position of these route variables will be insert into the
/// `RouteMap` with the variable's name (including the leading `:`).
///
/// If there are multiple routing variables in the same routing
/// specification, then this will `panic!` when resolving.
pub struct StandardRouter;

/// A resolver which follows the standard resolving method
/// described by [StandardRouter].
pub struct StandardResolver
{
    /// The routing specification
    route: Vec<String>
}

impl StandardRouter
{
    pub fn new() -> Self
    {
        StandardRouter {}
    }
}

impl Router for StandardRouter
{
    fn resolver(&self, route_spec: String) -> Box<RouteResolver>
    {
        // route specs are guaranteed to never have a leading or trailing /
        let route = route_spec.split("/")
            .map(String::from)
            // treat empty strings as meaning `None`, and remove them
            .filter_map(|it| {
                if it.is_empty() {
                    None
                }
                else {
                    Some(it)
                }
            })
            .collect();

        let resolver = StandardResolver {
            route
        };
        Box::new(resolver)
    }
}

impl RouteResolver for StandardResolver
{
    fn resolve(&self, route: &Vec<&str>) -> Option<RouteMap>
    {
        // if the number of parts to the route isn't correct, then we
        // should reject it
        if self.route.len() != route.len() {
            return None;
        }

        let mut map = RouteMap::new();
        for i in 0..route.len() {
            let expected = self.route[i].clone();
            let actual = route[i];

            // if this is a routing variable, then we'll always accept it
            if expected.starts_with(":") {
                if let Some(_) = map.insert(expected, actual.to_string()) {
                    panic!(
                        "Multiple routing variables with name {}",
                        self.route[i].clone()
                    );
                };
            }
            // otherwise, if the routing path literal didn't match, then we can
            // immediately reject
            else if *actual != *expected {
                return None;
            }
        }

        // we'll accept this route with the related variables' values
        Some(map)
    }
}

#[cfg(test)]
mod test
{
    use routing::*;

    #[test]
    fn standard_matches_root()
    {
        let router = StandardRouter::new();
        let resolver = router.resolver("".to_owned());
        assert!(resolver.resolve(&vec![]).is_some());
        assert!(resolver.resolve(&vec!["test"]).is_none());
        assert!(resolver.resolve(&vec!["test", "2"]).is_none());
    }

    #[test]
    fn standard_matches_literals()
    {
        let router = StandardRouter::new();
        let resolver = router.resolver("test".to_owned());
        assert!(resolver.resolve(&vec![]).is_none());
        assert!(resolver.resolve(&vec!["test"]).is_some());
        assert!(resolver.resolve(&vec!["test", "2"]).is_none());
    }

    #[test]
    fn standard_matches_rvars()
    {
        let router = StandardRouter::new();
        let resolver = router.resolver(":test".to_owned());
        assert!(resolver.resolve(&vec![]).is_none());
        assert!(resolver.resolve(&vec!["test", "2"]).is_none());

        match resolver.resolve(&vec!["aaa"]) {
            None => assert!(false),
            Some(args) => {
                assert_eq!(1, args.len());
                assert_eq!(Some(&"aaa".to_owned()), args.get(":test"));
            }
        }
    }

    #[test]
    fn standard_matches_all()
    {
        let router = StandardRouter::new();
        let resolver = router.resolver("user/:name/:action".to_owned());

        assert!(resolver.resolve(&vec![]).is_none());
        assert!(resolver.resolve(
                &vec!["files", "bad", "badstuff.zip"]
        ).is_none());

        match resolver.resolve(&vec!["user", "austin", "edit"]) {
            None => assert!(false),
            Some(args) => {
                assert_eq!(2, args.len());
                assert_eq!("austin", args[":name"]);
                assert_eq!("edit", args[":action"]);
            }
        }
    }
}
