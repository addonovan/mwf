use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use mime_guess;
use mime_guess::Mime;

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
        let path: PathBuf = file.into();
        let mut file = File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mime: Mime = mime_guess::guess_mime_type_opt(path)
            .unwrap_or_else(|| "text/plain".parse().unwrap());

        Ok(View {
            content,
            mime,
        })
    }

    /// Applies the given `decorator` to this view, consuming it and
    /// creating another one.
    pub fn apply<T: Decorator>(self, decorator: &T) -> Self
    {
        decorator.decorate(self)
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    /// Tests the [View::raw] API's ability to take in a `&'static str`
    #[test]
    fn from_raw_str()
    {
        // &'static str => View
        let view = View::raw("foobar");
        assert_eq!("foobar", view.content);
        assert_eq!("text", view.mime.type_());
        assert_eq!("plain", view.mime.subtype());
    }

    /// Tests the [View::raw] API's ability to take a `String`.
    #[test]
    fn from_string()
    {
        // String => View
        let view = View::raw("foobar".to_string());
        assert_eq!("foobar", view.content);

        // mime type testing by from_raw_str
    }

    /// Test the [View::file] API's ability to read a source file correctly.
    #[test]
    fn from_file()
    {
        // test reading an existing files
        let contents = include_str!("view.rs");
        let view = View::file("src/view.rs")
            .expect("Could not find or open src/view.rs for read");

        assert_eq!(contents, view.content);
    }

    /// Tests the [View::file] API's handling of IO errors while reading a
    /// file.
    #[test]
    fn from_nonexisting_file()
    {
        assert!(View::file("src/rs.view").is_err());
    }

    /// Tests the [View::file] API's correct detection of mime types.
    #[test]
    fn from_correct_mime_type()
    {
        let view = View::file("src/view.rs")
            .expect("Could not find or open src/view.rs for read");

        assert_eq!("text", view.mime.type_());
        assert_eq!("x-rust", view.mime.subtype());
    }

    // apply has been tested in the decorators files
    // no need to test it here too
}
