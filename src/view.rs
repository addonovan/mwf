use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use hyper::mime::Mime;

use error::Result;

pub struct View
{
    pub content: String,
    pub mime: Mime,
}

pub trait Decorator
{
    fn decorate(&self, view: View) -> View;
}

pub struct MarkdownDecorator;

//
// Implementation
//

impl View
{
    pub fn raw<T: Into<String>>(content: T) -> Self
    {
        View {
            content: content.into(),
            mime: "text/plain".parse().unwrap(),
        }
    }

    pub fn file<T: Into<PathBuf>>(file: T) -> Result<Self>
    {
        let mut file = File::open(file.into())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(View {
            content,
            mime: "text/plain".parse().unwrap(),
        })
    }

    pub fn apply<T: Decorator>(self, decorator: &T) -> Self
    {
        decorator.decorate(self)
    }
}

impl Decorator for MarkdownDecorator
{
    fn decorate(&self, view: View) -> View
    {
        use pulldown_cmark::{Parser, html};

        let mut output = String::new();
        let p = Parser::new(&view.content);
        html::push_html(&mut output, p);

        // create a new view with the html output and the correct
        // mime type
        View {
            content: output,
            mime: "text/html".parse().unwrap(),
        }
    }
}

