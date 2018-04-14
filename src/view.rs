use std::fs::File;
use std::convert::From;
use std::path::{Path, PathBuf};

use std::io;
use std::io::prelude::*;
use std::error::Error;

pub type ViewResult = Result<View, Box<Error>>;

/**
 * A view for the website.
 */
pub struct View
{
    content: String
}

impl View
{
    pub fn path<T: Into<PathBuf>>(path: T) -> ViewResult
    {
        let mut file = File::open(path.into().as_path())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(View {
            content
        })
    }

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

//
// Automatic conversions to View
//

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

