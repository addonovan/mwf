extern crate mwf;

use mwf::{ServerBuilder, View, ViewResult, RequestHandler};
use mwf::routing::RouteMap;

/// A simple structure which will just display the `markdown.md` file in this
/// same directory.
struct Markdown;
impl RequestHandler for Markdown
{
    fn handle(&self, _route_map: RouteMap) -> ViewResult
    {
        View::file("examples/markdown.md")
    }
}

fn main()
{
    ServerBuilder::new()
        .bind("/", Markdown {})
        .start()
        .unwrap();
}
