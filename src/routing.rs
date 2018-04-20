use hyper::{Method, Request};

use resolution::*;
use request_handler::RequestHandler;
use view::View;
use error::Result;

/// A function which will create a new [Resolver] instance.
type ResolverConstructor = Fn(Method, Vec<String>) -> Box<Resolver>;

/// An entry in the [Router]/[RouterBuilder]'s resolver vector.
///
/// Really, it's nothing more than a named tuple.
struct ResolverEntry
{
    pub resolver: Box<Resolver>,
    pub handler: Box<RequestHandler>,
}

/// Helps construct a thread-safe [Router] by using non-thread-safe operations
/// (such as creating new [Resolvers](Resolver)), until the server is spawned,
/// at which case this will be converted into a router.
pub struct RouterBuilder
{
    constructor: Box<ResolverConstructor>,
    resolvers: Vec<ResolverEntry>,
}

/// A thread-safe list of all [Resolvers](Resolver) and their corresponding
/// [RequestHandlers](RequestHandler). Also implements the normalization and
/// splitting of the path before the resolvers get to see it.
pub struct Router
{
    resolvers: Vec<ResolverEntry>,
}

//
// Implementation
//

impl Router
{
    /// Tries to handle the given `request`. If no resolvers accept the route
    /// then it will return `None`, indicating an Http Status 404.
    pub fn handle(&self, request: Request) -> Option<Result<View>>
    {
        let method = request.method();
        let route: Vec<&str> = request.path()
            .split("/")
            .filter_map(|it| {
                if it.is_empty() {
                    None
                }
                else {
                    Some(it)
                }
            })
            .collect();

        let params = ResolveParams {
            method: method.clone(),
            route,
        };

        for entry in &self.resolvers {
            let data = match entry.resolver.resolve(&params) {
                None => continue,
                Some(x) => x,
            };

            return Some(entry.handler.handle(data));
        }

        None
    }
}

impl RouterBuilder
{
    /// Constructs a new router builder which uses the [StandardResolver] by
    /// default.
    pub fn new() -> RouterBuilder
    {
        RouterBuilder {
            constructor: Box::new(StandardResolver::new),
            resolvers: Vec::new(),
        }
    }

    /// Changes the current resolver `constructor` for the given one. Every
    /// paged bound from now on will use this resolver instead.
    pub fn constructor(&mut self, constructor: Box<ResolverConstructor>)
    {
        self.constructor = constructor;
    }

    /// Binds a new request `handler` to the given route `spec` and connection
    /// `method`.
    pub fn bind<T: Into<String>, H: 'static>(
        &mut self,
        method: Method,
        spec: T,
        handler: H
    )
        where H: RequestHandler
    {
        let spec: String = spec.into();

        let spec: Vec<String> = spec.split("/")
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


        let constructor = &self.constructor;
        self.resolvers.push(
            ResolverEntry::new(
                constructor(method, spec),
                handler
            )
        );
    }
}

impl Into<Router> for RouterBuilder
{
    fn into(self) -> Router
    {
        Router {
            resolvers: self.resolvers,
        }
    }
}

impl ResolverEntry
{
    pub fn new<H: 'static>(resolver: Box<Resolver>, handler: H) -> Self
        where H: RequestHandler
    {
        ResolverEntry {
            resolver,
            handler: Box::new(handler),
        }
    }
}
