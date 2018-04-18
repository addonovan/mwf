extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, View, ViewResult};
use mwf::routing::RouteMap;

struct HelloWorld;
impl RequestHandler for HelloWorld
{
    fn handle(&self, _args: RouteMap) -> ViewResult
    {
        View::from("Hello, world!")
    }
}

/// The simplest server you can make with `mwf`.
///
/// This will use the default settings for everything, except
/// the root page, which will display a simple "Hello, world!"
/// message
fn main()
{
    ServerBuilder::new()
        .bind("/", HelloWorld {})
        .start()
        .unwrap();
}
