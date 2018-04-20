extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, RouteMap, View};

/// A simple request handler which will always reply with "Hello world"
/// any time it's asked for a response.
struct HelloWorld;
impl RequestHandler for HelloWorld
{
    fn handle(&self, _route_map: RouteMap) -> mwf::Result<View>
    {
        Ok(View::raw("Hello world!"))
    }
}

fn main()
{
    // We register the HelloWorld handler to respond to the `/` request
    // (i.e. the root)
    // The server, by default, will run on 127.0.0.1:8080
    ServerBuilder::new()
        .bind("/", HelloWorld)
        .start();
}
