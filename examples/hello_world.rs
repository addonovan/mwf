extern crate mwf;

use mwf::{View, ServerBuilder};

/// The simplest server you can make with `mwf`.
///
/// This will use the default settings for everything, except
/// the root page, which will display a simple "Hello, world!"
/// message
fn main()
{
    // notice that the pages are actually returning `Result<View>`s
    // and that From<&str> and From<String> are implemented for `View`
    //
    // View also has a method which takes anything which has `Into<View>`
    // implemented, and will simply wrap it in an `Ok`, which is the
    // actual method you are seeing invoked here

    ServerBuilder::new()
        // if the root page is requested, we'll respond with hello world!
        .on_page("/", |_| {
            View::from("Hello, world!")
        })
        // if any other page is requested, say goodbye, world!
        .on_page_not_found( |_| {
            View::from("Goodbye, world!")
        })
        .start()
        .unwrap();
}
