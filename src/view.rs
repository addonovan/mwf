use std::fs::File;
use std::convert::From;
use std::path::PathBuf;

use std::io::prelude::*;
use std::error::Error;

use iron::mime::Mime;

pub type ViewResult = Result<View, Box<Error>>;

/// A view for the website. This is simply something
/// which evaluates to a string.
pub struct View
{
    content: String,
    mime: Mime,
}

impl View
{
    /// Constructs a new view with the given `content` and the mime type
    /// "text/plain".
    ///
    /// You should avoid using this directly, if one of the `into()` methods
    /// applies.
    fn new(content: String) -> View
    {
        View {
            content,
            mime: "text/plain".parse().unwrap(),
        }
    }

    /// Updates the mime type to the given [mime].
    pub fn mime(&mut self, mime: Mime)
    {
        self.mime = mime;
    }
}

// Convenience Methods for View construction
impl View
{
    /// Attempts to read the file described by the given `path`.
    pub fn path<T: Into<PathBuf>>(path: T) -> ViewResult
    {
        let path = path.into();
        let path = path.as_path();

        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // the contents of the view will, for sure, be the contents of the
        // file. But the mimetype might change, so we have to be mutable
        let mut view: View = content.into();

        // check the extension on the file, if it's a valid html-like
        // extension, then we'll display it as html, otherwise, we'll
        // display it as plain text
        if let Some(ext) = path.extension() {

            // convert the extension to lowercase, that way we can be case
            // insensitive when checking if it's an html file.
            let ext = ext.to_str().unwrap().to_ascii_lowercase();
            if ext == "html" || ext == "htm" {
                view.mime("text/html".parse().unwrap());
            }
        }

        Ok(view)
    }

    /// Converts anything which can be converted `Into` a `View`, and
    /// simply runs its `into` method, then returns an `Ok` for the
    /// [ViewResult].
    ///
    /// Literally it's just `Ok(content.into())`
    pub fn from<T: Into<View>>(content: T) -> ViewResult
    {
        Ok(content.into())
    }
}

impl Into<(String, Mime)> for View
{
    fn into(self) -> (String, Mime)
    {
        (self.content, self.mime)
    }
}

impl From<&'static str> for View
{
    fn from(content: &str) -> Self
    {
        View::new(content.to_owned())
    }
}

impl From<String> for View
{
    fn from(content: String) -> Self
    {
        View::new(content)
    }
}

#[cfg(test)]
mod test
{
    use view::*;
    use iron::mime::{Mime, TopLevel, SubLevel};

    #[test]
    fn from_path()
    {
        let path = "src/view.rs";
        let expected = include_str!("view.rs");
        let expected = expected.to_owned();

        let view = View::path(path).unwrap();
        let (content, mime) = view.into();
        assert_eq!(expected, content);

        // make sure the mime type matches text/plain
        match mime {
            Mime(TopLevel::Text, SubLevel::Plain, _) => {},
            _ => {
                assert!(false);
            }
        }

        assert!(View::path("src/rs.view").is_err());
    }

    #[test]
    fn from_string()
    {
        let input = "a";
        let expected = input.clone();

        let view = View::from(input).unwrap();
        let (content, _) = view.into();
        assert_eq!(expected, content);
    }

    #[test]
    fn from_str()
    {
        let input = "a".to_owned();
        let expected = input.clone();

        let view = View::from(input).unwrap();
        let (content, _) = view.into();
        assert_eq!(expected, content);
    }

    #[test]
    fn into_tuple()
    {
        let input = "a".to_owned();
        let expected = input.clone();

        let view = View::from(input).unwrap();
        let (content, mime) = view.into();

        assert_eq!(expected, content);
        match mime {
            Mime(TopLevel::Text, SubLevel::Plain, _) => {},
            _ => {
                assert!(false);
            }
        }
    }
}
