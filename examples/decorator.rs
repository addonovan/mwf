extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, ViewResult, View, ViewDecorator};
use mwf::routing::RouteMap;

struct Decorator
{
    pre: String,
    post: String,
}

struct Markdown
{
    decorator: Decorator,
}

fn main()
{
    ServerBuilder::new()
        .bind("/", Markdown::new())
        .start()
        .unwrap();
}

impl Markdown
{
    fn new() -> Self
    {
        Markdown {
            decorator: Decorator {
                pre: "<!DOCTYPE html><html><body><div>Header!</div>".to_string(),
                post: "<div>Footer!</div></body></html>".to_string()
            }
        }
    }
}

impl RequestHandler for Markdown
{
    fn handle(&self, _route_map: RouteMap) -> ViewResult
    {
        View::file("examples/markdown.md")
            .and_then(|view| {
                self.decorator.decorate(view)
            })
    }
}

impl ViewDecorator for Decorator
{
    fn decorate(&self, view: View) -> ViewResult
    {
        let (content, mime) = view.into();
        let content = format!("{}{}{}", self.pre, content, self.post);

        View::from(content).and_then(|mut view| {
            view.mime(mime);
            Ok(view)
        })
    }
}
