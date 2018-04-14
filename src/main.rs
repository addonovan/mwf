extern crate iron;

mod mwf;
mod routing;
use mwf::{WebFrameworkBuilder};

fn main() {
    WebFrameworkBuilder::new()
        .on_page("/", |_| {
            "This is the root page!".to_owned()
        })
        .on_page("/test", |_| {
            "Hello world!".to_owned()
        })
        .on_page("/user/:name", |args| {
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
