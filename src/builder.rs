use std::sync::Arc;
use std::net::SocketAddr;

use hyper::server::Http;
use hyper::Method;

use routing::*;
use server::*;
use request_handler::RequestHandler;

pub enum Protocol
{
    Http,
}

pub struct ServerBuilder
{
    router: RouterBuilder,
    proto: Protocol,
    addr: SocketAddr,
}

impl ServerBuilder
{
    pub fn new() -> Self
    {
        ServerBuilder {
            router: RouterBuilder::new(),
            proto: Protocol::Http,
            addr: "127.0.0.1:8080".parse().unwrap(),
        }
    }

    pub fn bind<T: Into<String>, H: 'static>(
        mut self,
        route: T,
        handler: H
    ) -> Self
        where H: RequestHandler
    {
        self.router.bind(Method::Get, route, handler);
        self
    }

    pub fn addr(mut self, addr: SocketAddr) -> Self
    {
        self.addr = addr;
        self
    }

    pub fn start(self)
    {
        let router: Arc<Router> = Arc::new(self.router.into());

        let server = Http::new().bind(&self.addr, move || {
            let router = router.clone();
            Ok(Server::new(router))
        }).unwrap();

        server.run().unwrap();
    }
}
