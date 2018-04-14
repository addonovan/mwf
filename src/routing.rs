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
        StandardRouter {

        }
    }
}

impl Router for StandardRouter
{
    fn resolver(&self, route_spec: String) -> Box<RouteResolver>
    {
        let route = route_spec.split("/")
            .skip(1) // skip the leading root
            .map(String::from)
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
