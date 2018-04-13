extern crate iron;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use iron::*;
use iron::prelude::*;
use iron::Error;
use iron::error::HttpResult;
use iron::status;
use iron::Request;

type PageHandler = fn(&Vec<&str>) -> String;

//
// The framework builder
//

pub struct WebFrameworkBuilder
{
    pages: HashMap<String, PageHandler>,
}

impl WebFrameworkBuilder
{
    pub fn new() -> Self
    {
        WebFrameworkBuilder {
            pages: HashMap::new(),
        }
    }

    pub fn on_page(mut self, path: String, handler: PageHandler) -> Self
    {
        self.pages.insert(path, handler);
        self
    }

    pub fn start(self) -> HttpResult<Listening>
    {
        let framework = WebFramework::new(self.pages);
        let framework = RwLock::new(framework);
        let framework = Arc::new(framework);

        let call = move | m: &mut Request | {
            let framework = framework.clone();
            let framework = framework.read().unwrap();
            framework.handle(m)
        };

        Iron::new(call).http("localhost:8080")
    }
}

//
// The framework itself
//

struct WebFramework
{
    pages: HashMap< String, PageHandler >,
}

impl WebFramework
{
    fn new(pages: HashMap<String, PageHandler>) -> Self
    {
        WebFramework {
            pages,
        }
    }

    fn handle(&self, request: &mut Request) -> IronResult<Response>
    {
        let path = request.url.path();
        let seg = path.get(0).unwrap();

        if let Some(handler) = self.pages.get::<str>(&seg) {
            let response = handler(&path);
            Ok(Response::with((status::Ok, response)))
        }
        else {
            Ok(Response::with(status::NotFound))
        }
    }
}

