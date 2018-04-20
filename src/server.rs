use std::sync::Arc;

use futures;
use futures::Future;

use hyper;
use hyper::server::{Request, Response, Service};
use hyper::StatusCode;
use hyper::header::ContentType;

use routing::Router;

/// The basic server service which is used to try to resolve paths
/// and respond with the correct information.
pub struct Server
{
    router: Arc<Router>,
}

impl Server
{
    /// Creates a new instance of the server service, which simply tries
    /// uses the given `router` to find a page.
    pub fn new(router: Arc<Router>) -> Self
    {
        Server {
            router,
        }
    }
}

impl Service for Server
{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future
    {
        let response = match self.router.handle(req) {

            // No response => 404
            None => {
                let mut response = Response::new();
                response.set_status(StatusCode::NotFound);
                response.set_body("404\nRequested file not found");
                response
            },

            // We found something, so use that as our body!
            Some(result) => {
                let mut response = Response::new();

                match result {
                    Err(error) => {
                        response.set_body("Internal Server Error");
                        response.set_status(StatusCode::InternalServerError);
                        println!("{}", error);
                    },

                    Ok(view) => {
                        response.set_body(view.content);
                        response.headers_mut().set(ContentType(view.mime));
                    }
                }

                response
            }
        };

        Box::new(futures::future::ok(response))
    }
}
