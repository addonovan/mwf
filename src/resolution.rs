use std::collections::HashMap;

use hyper::server::Request;
use hyper::Method;

pub type RouteMap = HashMap<String, String>;

pub struct ResolveParams<'a>
{
    pub method: Method,
    pub route: Vec<&'a str>,
}

pub trait Resolver
    where Self: Send + Sync
{
    fn resolve(&self, params: &ResolveParams) -> Option<RouteMap>;
}

//
// Standard Resolver
//

pub enum Token
{
    Literal(String),
    Variable(String),
    Optional(String),
}

pub struct StandardResolver
{
    method: Method,
    spec: Vec<Token>,
}

impl StandardResolver
{
    pub fn new(method: Method, spec: Vec<String>) -> Box<Resolver>
    {
        let spec = spec.into_iter()
            .map(|token| {
                if token.starts_with(":") {
                    if token.ends_with("?") {
                        Token::Optional(token)
                    }
                    else {
                        Token::Variable(token)
                    }
                }
                else {
                    Token::Literal(token)
                }
            })
            .collect();

        Box::new(StandardResolver {
            method,
            spec,
        })
    }
}

impl Resolver for StandardResolver
{
    fn resolve(&self, params: &ResolveParams) -> Option<RouteMap>
    {
        // resolution MUST have the same request method
        if params.method != self.method {
            return None;
        }

        let mut map = RouteMap::new();
        let mut i = 0;

        while let Some(expected) = self.spec.get(i) {
            let actual = params.route.get(i);

            match expected {
                &Token::Literal(ref expected) => {
                    let actual = actual?;
                    if actual != &expected.as_str() {
                        return None;
                    }
                },

                &Token::Variable(ref name) => {
                    let actual = actual.map(|x| x.to_string())?;
                    let name = name.clone();

                    if let Some(_) = map.insert(name, actual) {
                        panic!("Multiple variables with the same name!");
                    }
                },

                &Token::Optional(ref name) => {
                    let text: String = match actual {
                        None => "".into(),
                        Some(x) => x.to_string(),
                    };
                    let name = name.clone();

                    if let Some(_) = map.insert(name, text) {
                        panic!("Multiple variables with the same name!");
                    }
                }
            }

            i += 1;
        }

        // if we still have more route to match, then we can't match
        if i < params.route.len() {
            None
        }
        else {
            Some(map)
        }
    }
}

