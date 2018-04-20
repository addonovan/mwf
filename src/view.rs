use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use hyper::mime::Mime;

use error::Result;
use decorator::Decorator;

/// A view on the server.
pub struct View
{
    /// The content to display
    pub content: String,

    /// The contents mime type
    pub mime: Mime,
}

//
// Implementation
//

impl View
{
    /// Constructs a view from the raw text in `content`.
    /// This will have the `text/plain` mime type.
    pub fn raw<T: Into<String>>(content: T) -> Self
    {
        View {
            content: content.into(),
            mime: "text/plain".parse().unwrap(),
        }
    }

    /// Constructs a view from the text in the given `file`.
    /// This will have the `text/plain` mime type.
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

    /// Applies the given `decorator` to this view, consuming it and
    /// creating another one.
    pub fn apply<T: Decorator>(self, decorator: &T) -> Self
    {
        decorator.decorate(self)
    }
}
