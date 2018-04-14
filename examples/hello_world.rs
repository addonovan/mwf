extern crate mwf;

use mwf::{View, ServerBuilder};

/// The simplest server you can make with `mwf`.
///
/// This will use the default settings for everything, except
/// the root page, which will display a simple "Hello, world!"
/// message
fn main()
{
    ServerBuilder::new()
        .on_page("/", |_| {
            View::from("Hello, world!")
        })
        .start()
        .unwrap();
}
