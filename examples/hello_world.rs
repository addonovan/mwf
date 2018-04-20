extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, RouteMap};

struct HelloWorld;
impl RequestHandler for HelloWorld
{
    fn handle(&self, route_map: RouteMap) -> String
    {
        "Hello World!".into()
    }
}

fn main()
{
    ServerBuilder::new()
        .bind("/", HelloWorld)
        .start();
}


