extern crate iron;

mod mwf;
use mwf::{WebFrameworkBuilder};

fn main() {
    WebFrameworkBuilder::new()
        .on_page("test".to_owned(), |_| {
            "Hello world!".to_owned()
        })
        .on_page("user/:name".to_owned(), |args| {
            format!( "hello, {}!", args[":name"])
        })
        .start()
        .unwrap();
}
