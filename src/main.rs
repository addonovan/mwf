extern crate iron;

use iron::IronResult;
use iron::Response;
use iron::Request;
use iron::status;
use iron::prelude::Iron;

fn hello_world(_: &mut Request) -> IronResult< Response >
{
    Ok( Response::with( ( status::Ok, "Hello world!" ) ) )
}

fn main() {
    let _server = Iron::new(hello_world)
            .http("localhost:8080")
            .unwrap();
    println!("Running");
}
