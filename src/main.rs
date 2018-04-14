extern crate iron;

mod mwf;
mod routing;
use mwf::{WebFrameworkBuilder};
use routing::StandardRouter;

fn main() {
    WebFrameworkBuilder::new()
        .router(StandardRouter::new())
        .on_page("test", |_| {
            "Hello world!".to_owned()
        })
        .on_page("user/:name", |args| {
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
