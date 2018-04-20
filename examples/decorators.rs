extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, RouteMap, View};
use mwf::decorator;

struct DecoratorExample
{
    markdown: decorator::Markdown,
    page: decorator::Surround,
}

impl DecoratorExample
{
    pub fn new() -> Self
    {
        DecoratorExample {
            markdown: decorator::Markdown,
            page: decorator::Surround::new(
                "<!DOCTYPE html><html><body><div>I'm a header</div><article>",
                "</article><div>I'm a footer</div></body></html>"
            ),
        }
    }
}

impl RequestHandler for DecoratorExample
{
    fn handle(&self, _route_map: RouteMap) -> mwf::Result<View>
    {
        Ok(
            View::raw("# Hello World!")
                .apply(&self.markdown)
                .apply(&self.page)
        )
    }
}

fn main()
{
    ServerBuilder::new()
        .bind("/", DecoratorExample::new())
        .start();
}
