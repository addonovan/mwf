use hyper::Method;

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
