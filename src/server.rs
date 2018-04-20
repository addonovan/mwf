
use futures;
use futures::Future;

use hyper;
use hyper::header::ContentLength;
use hyper::server::{Request, Response, Service};
use hyper::{Method, StatusCode};

pub struct Server
{

}

impl Service for Server
{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future
    {
        let mut response = Response::new();
        response.set_body("Hello, world!");

        Box::new(futures::future::ok(response))
    }
}
