extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, RouteMap, View};

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
    ServerBuilder::new()
        .bind("/", HelloWorld)
        .start();
}
