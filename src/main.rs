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
        .on_page_not_found(|args| {
            format!(
                "404 Not Found\n`{}` could not found found on this server",
                args["path"]
            )
        })
        .start()
        .unwrap();
}
