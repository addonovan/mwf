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
    /// Attempts to read the file described by the given `path`.
    pub fn path<T: Into<PathBuf>>(path: T) -> ViewResult
    {
        let path = path.into();
        let path = path.as_path();

        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // check the extension on the file, if it's a valid html-like
        // extension, then we'll display it as html, otherwise, we'll
        // display it as plain text
        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap().to_ascii_lowercase();
            if ext == "html" || ext == "htm" {
                return Ok(View {
                    content,
                    mime: "text/html".parse().unwrap(),
                });
            }
        }

        View::from(content)
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
        View {
            content: content.to_owned(),
            mime: "text/plain".parse().unwrap(),
        }
    }
}

impl From<String> for View
{
    fn from(content: String) -> Self
    {
        View {
            content,
            mime: "text/plain".parse().unwrap(),
        }
    }
}

#[cfg(test)]
mod test
{
    use view::*;

    #[test]
    fn from_path()
    {
        let path = "src/view.rs";
        let expected = include_str!("view.rs");
        let expected = expected.to_owned();

        let view = View::path(path).unwrap();
        assert_eq!(expected, view.content);

        assert!(View::path("src/rs.view").is_err());
    }

    #[test]
    fn from_string()
    {
        let input = "a";
        let expected = input.to_owned();
        assert_eq!(expected, View::from(input).unwrap().content);
    }

    #[test]
    fn from_str()
    {
        let input = "a".to_owned();
        let expected = input.clone();
        assert_eq!(expected, View::from(input).unwrap().content);
    }

    #[test]
    fn into_string()
    {
        let input = "a".to_owned();
        let expected = input.clone();
        let view: String = View::from(input).unwrap().into();
        assert_eq!(expected, view);
    }
}
