use std::fs::File;
use std::convert::From;
use std::path::PathBuf;

use std::io::prelude::*;
use std::error::Error;

pub type ViewResult = Result<View, Box<Error>>;

/// A view for the website. This is simply something
/// which evaluates to a string.
pub struct View
{
    content: String
}

impl View
{
    /// Attempts to read the file described by the given `path`.
    pub fn path<T: Into<PathBuf>>(path: T) -> ViewResult
    {
        let mut file = File::open(path.into().as_path())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(View {
            content
        })
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

impl Into<String> for View
{
    fn into(self) -> String
    {
        self.content
    }
}

impl From<&'static str> for View
{
    fn from(content: &str) -> Self
    {
        View {
            content: content.to_owned()
        }
    }
}

impl From<String> for View
{
    fn from(content: String) -> Self
    {
        View {
            content
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
