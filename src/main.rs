extern crate iron;

mod mwf;
mod routing;
mod view;

use std::convert::From;

use view::View;
use mwf::{WebFrameworkBuilder};

fn main() {
    WebFrameworkBuilder::new()
        .on_page("/", |_| {
            View::from("This is the root page!")
        })
        .on_page("/test", |_| {
            View::from("Hello, world!")
        })
        .on_page("/user/:name", |args| {
            View::from(format!("hello, {}!", args[":name"]))
        })
        .on_page("/file/:name", |args| {
            let file_name = args[":name"].clone();
            View::path(file_name)
        })
        .on_page_not_found(|_| {
            View::from("404 not found :(")
        })
        .start()
        .unwrap();
}
