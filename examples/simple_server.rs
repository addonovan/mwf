extern crate iron;
extern crate mwf;

use mwf::{View, ServerBuilder};

fn main() {
    ServerBuilder::new()
        .on_page("/", |_| {
            View::from("This is the root page!")
        })
        .on_page("/test", |_| {
            View::from("Hello, world!")
        })
        .on_page("/user/:name", |args| {
            let user_name = &args[":name"];
            let text = format!("hello, {}!", user_name);
            View::from(text)
        })
        .on_page("/file/:name", |args| {
            let file_name = &args[":name"];
            View::path(file_name)
        })
        .on_page_not_found(|_| {
            View::from(
                r#"
                    OOPSIE WOOPSIE!!
                    Uwu We made a fucky wucky!!
                    A wittle fucko boingo!
                    The code monkeys at our headquarters are working VEWY HAWD to fix this!
                "#
            )
        })
        .start()
        .unwrap();
}
