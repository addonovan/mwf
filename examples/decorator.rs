extern crate mwf;

use mwf::{ServerBuilder, RequestHandler, ViewResult, View, ViewDecorator};
use mwf::routing::RouteMap;

/// This decorator will prepend `pre` and append `post` onto its input
/// view's content.
struct Decorator
{
    /// the string to prepend onto the front of the view's content
    pre: String,

    /// the string to append onto the end of the view's content
    post: String,
}

/// A simple structure which will decorate the `markdown.md` file by
/// using the [Decorator] struct.
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
    /// Creates a new markdown struct which will use a decorate which will:
    /// prepend:
    /// ```html
    /// <!DOCTYPE html>
    /// <html>
    /// <body>
    /// <div>Header!</div>
    /// ```
    ///
    /// and append:
    /// ```html
    /// <div>Footer!</div>
    /// </body>
    /// </html>
    /// ```
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
        // view the file, then decorate the resultant view (if it succeeded)
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
        // surround the view's content with our `pre` and `post` members
        let (content, mime) = view.into();
        let content = format!("{}{}{}", self.pre, content, self.post);

        // create a new view from our content, and make sure it keeps the
        // same mime type as before!
        View::from(content).and_then(|mut view| {
            view.mime(mime);
            Ok(view)
        })
    }
}
