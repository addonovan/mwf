extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, RouteMap, View};
use mwf::decorator;

/// This is what our page should be formatted like.
/// The `{{middle}}` part is a delimiter to tell `decorator::Surround` where it
/// needs to split the string to insert the view's content.
const PAGE_FORMAT: &'static str =
r#"
<!DOCTYPE html>
<html>
<body>
  <div>I'm a header!</div>
  <article> {{middle}} </article>
  <div>I'm a footer!</div>
</body>
</html>
"#;

/// A simple example of the decorator class.
///
/// This will use the [Markdown](decorator::Markdown) decorator to convert
/// markdown into html, then use the [Surround](decorator::Surround) decorator
/// to insert it into our page format.
///
/// This is a farily realistic example of something you might want to do. If you
/// write your website pages in markdown, then just use the view to transform
/// it to HTML, then insert it where the content in your site should go.
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
            page: decorator::Surround::from(PAGE_FORMAT),
        }
    }
}

impl RequestHandler for DecoratorExample
{
    fn handle(&self, _route_map: RouteMap) -> mwf::Result<View>
    {
        Ok(
            View::raw("# Hello World!") // given basic markdown text
                .apply(&self.markdown)  // convert it to html
                .apply(&self.page)      // and insert it into our page format
        )
    }
}

fn main()
{
    ServerBuilder::new()
        .bind("/", DecoratorExample::new())
        .start();
}
