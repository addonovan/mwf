extern crate iron;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use super::routing::*;
use super::view::*;

use iron::*;
use iron::error::HttpResult;
use iron::status;
use iron::Request;

///Generates a page based on the routing information in the [RouteMap]
pub type PageHandler = Box<Fn(RouteMap) -> ViewResult + Send + Sync>;

//
// The framework builder
//

/// A builder-pattern for constructing a new [Server].
pub struct ServerBuilder
{
    pages: HashMap<String, PageHandler>,
    page_not_found: PageHandler,
    router: Box<Router>,
    address: String,
}

impl ServerBuilder
{
    /// Creates a new builder with the following defualts:
    /// * No bound pages
    /// * 404 page reads "Page Not Found"
    /// * [StandardRouter]
    /// * Bound to `localhost:8080`
    pub fn new() -> Self
    {
        let page_not_found = |_| {
            View::from("Page Not Found")
        };
        let page_not_found = Box::new(page_not_found);

        ServerBuilder {
            pages: HashMap::new(),
            page_not_found,
            router: Box::new(StandardRouter::new()),
            address: "localhost:8080".to_owned(),
        }
    }

    /// Changes the [Router] instance to use to generate
    /// [RouteResolvers] once the web server is started.
    pub fn router<T: 'static + Router>(mut self, router: T) -> Self
    {
        self.router = Box::new(router);
        self
    }

    /// Binds a `handler` to the given route specification, `path`.
    pub fn on_page<F: 'static + Send + Sync>(
        mut self,
        path: &str,
        handler: F
    ) -> Self
        where F: Fn(RouteMap) -> ViewResult
    {
        // clean up the path first
        // remove leading and trailing slashes (as they aren't necessary)
        let path = path.trim_left_matches("/")
            .trim_right_matches("/");

        self.pages.insert(path.to_owned(), Box::new(handler));
        self
    }

    /// Binds a `handler` to the 404 page.
    pub fn on_page_not_found<F: 'static + Send + Sync>(
        mut self,
        handler: F
    ) -> Self
        where F: Fn(RouteMap) -> ViewResult
    {
        self.page_not_found = Box::new(handler);
        self
    }

    /// Specifies the new `addr` that the server will run on.
    pub fn address<T: Into<String>>(mut self, addr: T) -> Self
    {
        self.address = addr.into();
        self
    }

    /// Starts the http server described by this Server Builder.
    ///
    /// This is a blocking call.
    pub fn start(self) -> HttpResult<Listening>
    {
        let framework = Server::new(
            self.router,
            self.pages,
            self.page_not_found
        );
        let framework = RwLock::new(framework);
        let framework = Arc::new(framework);

        let call = move | m: &mut Request | {
            let framework = framework.clone();
            let framework = framework.read().unwrap();
            framework.handle(m)
        };

        Iron::new(call).http(self.address)
    }
}

//
// The framework itself
//

/// An instance of a running webserver.
struct Server
{
    pages: Vec<(Box<RouteResolver>, PageHandler)>,
    page_not_found: PageHandler,
}

impl Server
{
    /// Creates a new server, which uses `router` to generate `RouteResolvers`
    /// for the given `pages` (which is done before the server is started),
    /// and which will serve `page_not_found` as its 404 page.
    fn new(
        router: Box<Router>,
        pages: HashMap<String, PageHandler>,
        page_not_found: PageHandler,
    ) -> Self
    {
        // creates RouteResolvers for all of the route specifications
        let pages = pages.into_iter()
            .map(|(s, h)| {
                (router.resolver(s), h)
            })
            .collect();

        Server {
            pages,
            page_not_found,
        }
    }

    /// Handles an incoming `request`.
    fn handle(&self, request: &mut Request) -> IronResult<Response>
    {
        let route = request.url.path();

        for &(ref resolver, ref handler) in &self.pages {
            // see if this resolver successfully matches the given route
            let data = match resolver.resolve(&route) {
                None => continue,
                Some(x) => x,
            };

            // safely get the View from the handler
            return match handler(data) {
                Ok(content) => {
                    let content: String = content.into();
                    Ok(Response::with((status::Ok, content)))
                },

                Err(reason) => {
                    let reason: String = reason.to_string();
                    Ok(Response::with((status::InternalServerError, reason)))
                }
            }
        }

        // couldn't find a page that would accept it
        let mut map = HashMap::new();
        map.insert( "path".to_owned(), route.join("/").to_owned() );

        // default to the 404 page
        let handler = &self.page_not_found;
        match handler(map) {
            Ok(content) => {
                let content: String = content.into();
                Ok(Response::with((status::NotFound, content)))
            },

            Err(reason) => {
                let reason: String = reason.to_string();
                Ok(Response::with((status::InternalServerError, reason)))
            }
        }
    }
}
