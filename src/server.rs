extern crate iron;

use std::sync::{Arc, RwLock};

use routing::*;

use iron::*;
use iron::error::HttpResult;
use iron::status;
use iron::Request;
use iron::headers::ContentType;

use handle::RequestHandler;

//
// The framework builder
//

/// A builder-pattern for constructing a new [Server].
pub struct ServerBuilder
{
    router: RouterBuilder,
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
        ServerBuilder {
            router: RouterBuilder::new(),
            address: "localhost:8080".to_owned(),
        }
    }

    pub fn bind<T: Into<String>, H: 'static>(mut self, path: T, handler: H) -> Self
        where H: RequestHandler
    {
        self.router.bind(path, handler);
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
            self.router.into(),
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
    router: Router,
}

impl Server
{
    fn new(router: Router) -> Self
    {
        Server {
            router
        }
    }

    /// Handles an incoming `request`.
    fn handle(&self, request: &mut Request) -> IronResult<Response>
    {
        let result = match self.router.handle(request) {
            Some(x) => x,

            // page not found :(
            None => {
                return Ok(Response::with((status::NotFound, "Oh no :(")));
            }
        };

        match result {
            Ok(view) => {
                let (content, mime) = view.into();

                let mut response = Response::with((status::Ok, content));
                response.headers.set(ContentType(mime));

                Ok(response)
            },

            Err(reason) => {
                let reason = reason.to_string();
                Ok(Response::with((status::InternalServerError, reason)))
            }
        }
    }
}
