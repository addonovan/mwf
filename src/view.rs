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

