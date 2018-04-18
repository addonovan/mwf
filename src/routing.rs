extern crate iron;

use std::collections::HashMap;
use view::ViewResult;
use handle::RequestHandler;

/// Maps a route's variables to their respective values
pub type RouteMap = HashMap<String, String>;

/// Resolves a route.
pub trait Resolver
    where Self: Send + Sync
{
    /// Resolves a request `route` to returns a `RouteMap` binding
    /// any route-specific information to a key.
    fn resolve(&self, route: &Vec<&str>) -> Option<RouteMap>;
}

/// The constructor for a [Resolver].
type ResolverConstructor = Fn(Vec<String>) -> Box<Resolver>;

pub struct Router
{
    resolvers: Vec<(Box<Resolver>, Box<RequestHandler>)>,
}

pub struct RouterBuilder
{
    /// The constructor used to create new resolvers.
    constructor: Box<ResolverConstructor>,

    resolvers: Vec<(Box<Resolver>, Box<RequestHandler>)>,
}

impl RouterBuilder
{
    pub fn new() -> RouterBuilder
    {
        RouterBuilder {
            constructor: Box::new(StandardResolver::new),
            resolvers: Vec::new(),
        }
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

/// A resolver which follows the standard resolving method
/// described by [StandardRouter].
pub struct StandardResolver
{
    /// The routing specification
    route: Vec<String>
}

impl StandardResolver
{
    fn new(route: Vec<String>) -> Box<Resolver>
    {
        Box::new(StandardResolver {
            route
        })
    }
}

impl Resolver for StandardResolver
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
