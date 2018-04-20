use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use hyper::mime::Mime;

use error::Result;
use decorator::Decorator;

pub struct View
{
    pub content: String,
    pub mime: Mime,
}

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
