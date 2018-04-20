use hyper::{Method, Request};

use resolution::*;
use request_handler::RequestHandler;

type ResolverConstructor = Fn(Method, Vec<String>) -> Box<Resolver>;

struct ResolverEntry
{
    pub resolver: Box<Resolver>,
    pub handler: Box<RequestHandler>,
}

pub struct RouterBuilder
{
    constructor: Box<ResolverConstructor>,
    resolvers: Vec<ResolverEntry>,
}

pub struct Router
{
    resolvers: Vec<ResolverEntry>,
}

//
// Implementation
//

impl Router
{
    pub fn handle(&self, request: Request) -> Option<String>
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
    pub fn new() -> RouterBuilder
    {
        RouterBuilder {
            constructor: Box::new(StandardResolver::new),
            resolvers: Vec::new(),
        }
    }

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
