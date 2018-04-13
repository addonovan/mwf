extern crate iron;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use iron::*;
use iron::error::HttpResult;
use iron::status;
use iron::Request;

pub type PageHandler = fn(HashMap<String, String>)-> String;

//
// The framework builder
//

pub struct WebFrameworkBuilder
{
    pages: HashMap<Vec<String>, PageHandler>,
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
        let path = path.split("/")
            .map(|it| it.to_owned())
            .collect();
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
    pages: Vec<(PathMatcher, PageHandler)>,
}

impl WebFramework
{
    fn new(pages: HashMap<Vec<String>, PageHandler>) -> Self
    {
        let pages = pages.into_iter().map(|it| {
            (PathMatcher::new(it.0), it.1)
        }).collect();

        WebFramework {
            pages
        }
    }

    fn handle(&self, request: &mut Request) -> IronResult<Response>
    {
        let path = request.url.path();

        for &(ref matcher, ref handler) in &self.pages {
            let data = match matcher.matches(&path) {
                None => continue,
                Some(x) => x,
            };

            let content = handler(data);
            return Ok(Response::with((status::Ok, content)));
        }

        Ok(Response::with(status::NotFound))
    }
}

struct PathMatcher
{
    parts: Vec<String>,
}

impl PathMatcher
{
    fn new(parts: Vec<String>) -> Self
    {
        PathMatcher {
            parts
        }
    }

    fn matches(&self, path: &Vec<&str>) -> Option<HashMap<String, String>>
    {
        if self.parts.len() != path.len() {
            return None;
        }

        let mut map = HashMap::new();
        for i in 0..self.parts.len() {
            let expected = self.parts[ i ].clone();
            let actual = path[ i ];

            if expected == *actual {
                continue;
            }
            else if expected.starts_with( ":" ) {
                map.insert(expected, actual.to_string());
            }
            else {
                return None;
            }
        }

        Some(map)
    }
}
