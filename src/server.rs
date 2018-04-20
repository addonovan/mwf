use std::sync::Arc;

use futures;
use futures::Future;

use hyper;
use hyper::server::{Request, Response, Service};

use routing::Router;
use hyper::StatusCode;

pub struct Server
{
    router: Arc<Router>,
}

impl Server
{
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
            None => {
                let mut response = Response::new();
                response.set_status(StatusCode::NotFound);
                response.set_body("404\nRequested file not found");
                response
            },

            Some(x) => {
                let mut response = Response::new();
                response.set_body(x);
                response
            }
        };

        Box::new(futures::future::ok(response))
    }
}
