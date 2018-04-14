extern crate iron;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use routing::*;
use view::*;

use iron::*;
use iron::error::HttpResult;
use iron::status;
use iron::Request;

pub type PageHandler = fn(RouteMap) -> ViewResult;

//
// The framework builder
//

pub struct WebFrameworkBuilder
{
    pages: HashMap<String, PageHandler>,
    page_not_found: PageHandler,
    router: Box<Router>,
}

impl WebFrameworkBuilder
{
    pub fn new() -> Self
    {
        let page_not_found = |_| {
            View::from("Page Not Found")
        };

        WebFrameworkBuilder {
            pages: HashMap::new(),
            page_not_found,
            router: Box::new(StandardRouter::new()),
        }
    }

    pub fn router<T: 'static + Router>(mut self, router: T) -> Self
    {
        self.router = Box::new(router);
        self
    }

    pub fn on_page(mut self, path: &str, handler: PageHandler) -> Self
    {
        self.pages.insert(path.to_owned(), handler);
        self
    }

    pub fn on_page_not_found(mut self, handler: PageHandler) -> Self
    {
        self.page_not_found = handler;
        self
    }

    pub fn start(self) -> HttpResult<Listening>
    {
        let framework = WebFramework::new(
            self.router,
            self.pages,
            self.page_not_found
        );
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
    pages: Vec<(Box<RouteResolver>, PageHandler)>,
    page_not_found: PageHandler,
}

impl WebFramework
{
    fn new(
        router: Box<Router>,
        pages: HashMap<String, PageHandler>,
        page_not_found: PageHandler,
    ) -> Self
    {
        let pages: Vec<(Box<RouteResolver>, PageHandler)> = pages.into_iter()
            .map(|(s, h)| {
                (router.resolver(s), h)
            })
            .collect();

        WebFramework {
            pages,
            page_not_found,
        }
    }

    fn handle(&self, request: &mut Request) -> IronResult<Response>
    {
        let path = request.url.path();

        for &(ref matcher, ref handler) in &self.pages {
            let data = match matcher.resolve(&path) {
                None => continue,
                Some(x) => x,
            };

            return match handler(data) {
                Ok(content) => {
                    let content: String = content.into();
                    Ok(Response::with((status::Ok, content)))
                },

                Err(reason) => {
                    let reason: String = reason.to_string();
                    Ok(Response::with((status::InternalServerError, reason)))
                }
            }
        }

        // couldn't find a page that would accept it
        let mut map = HashMap::new();
        map.insert( "path".to_owned(), path.join("/").to_owned() );

        let handler = self.page_not_found;
        match handler(map) {
            Ok(content) => {
                let content: String = content.into();
                Ok(Response::with((status::NotFound, content)))
            },

            Err(reason) => {
                let reason: String = reason.to_string();
                Ok(Response::with((status::InternalServerError, reason)))
            }
        }
    }
}
