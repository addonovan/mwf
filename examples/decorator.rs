extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, RouteMap, View, MarkdownDecorator};

struct DecoratorExample
{
    decorator: MarkdownDecorator,
}

impl DecoratorExample
{
    pub fn new() -> Self
    {
        DecoratorExample {
            decorator: MarkdownDecorator,
        }
    }
}

impl RequestHandler for DecoratorExample
{
    fn handle(&self, _route_map: RouteMap) -> mwf::Result<View>
    {
        Ok(View::raw("# Hello World!").apply(&self.decorator))
    }
}

fn main()
{
    ServerBuilder::new()
        .bind("/", DecoratorExample::new())
        .start();
}
