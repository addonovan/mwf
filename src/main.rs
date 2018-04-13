extern crate iron;

mod mwf;
use mwf::{WebFrameworkBuilder};

fn main() {
    WebFrameworkBuilder::new()
        .on_page("test".to_owned(), |path| {
            "Hello world!".to_owned()
        })
        .on_page("test2".to_owned(), |path| {
            "Goodbye world!".to_owned()
        })
        .start()
        .unwrap();
}
