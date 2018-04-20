use std::sync::Arc;
use std::net::SocketAddr;

use hyper::server::Http;
use hyper::Method;

use routing::*;
use server::*;
use request_handler::RequestHandler;

/// The protocol to use for the server.
pub enum Protocol
{
    Http,
}

/// The server building interface. This streamlines the entire process of
/// creating a server.
pub struct ServerBuilder
{
    router: RouterBuilder,
    proto: Protocol,
    addr: SocketAddr,
}

impl ServerBuilder
{
    /// Creates a new server build with the following defaults:
    /// * No routes set up
    /// * Served over HTTP
    /// * bound to `127.0.0.1:8080`
    pub fn new() -> Self
    {
        ServerBuilder {
            router: RouterBuilder::new(),
            proto: Protocol::Http,
            addr: "127.0.0.1:8080".parse().unwrap(),
        }
    }

    /// Binds a new `handler` to a given `route` on a GET request.
    /// See [on] for POST requests.
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

    /// Binds a new `handler` to a given `route` on a POST request.
    /// See [bind] for GET requests.
    pub fn on<T: Into<String>, H: 'static>(
        mut self,
        route: T,
        handler: H
    ) -> Self
        where H: RequestHandler
    {
        self.router.bind(Method::Post, route, handler);
        self
    }

    /// Binds the server to listen to a new `address`.
    pub fn addr(mut self, address: SocketAddr) -> Self
    {
        self.addr = address;
        self
    }

    /// Changes the `protocol` the server should use.
    pub fn proto(mut self, protocol: Protocol) -> Self
    {
        self.proto = protocol;
        self
    }

    /// Starts the server with the current configuration.
    /// This *will* panic if the server couldn't be started for some reason.
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
