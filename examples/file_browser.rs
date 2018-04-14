extern crate mwf;

use mwf::{View, ServerBuilder};
use mwf::routing::{Router, RouteResolver, RouteMap};

use std::path::PathBuf;

/// Routes files to a file handler.
///
/// A route is composed of tokens, which are delineated by `/'.
/// For a file router, there are the following tokens:
/// * path literal: matches the exact part of the path
/// * `*`: matches anything, including subdirectories and their
///   files.
///
/// A route specification can be composed of any number of path
/// literals, followed by no more than one `*`.
///
/// These are some examples of valid route specifications:
/// * `/index.html`: Matches `/index.html` exactly
/// * `/dir/index.html`: Matches `/dir/index.html` exactly
/// * `/dir/hello*`: Matches `/dir/hello*` exactly
/// * `/dir/*`: Matches all files and directories in `/dir/`
/// * `/*`: Matches all files and directories on the server
///
/// The full path name of the file that matched will be passed
/// to the [PageHandler] in the "file" entry of the [HashMap].
struct FileRouter;

/// Resolves only paths which match the [path] specification.
/// See [FileRouter] for a description of how the routes are
/// matched.
struct FileResolver
{
    path: Vec<String>,
}


/// The simplest server you can make with `mwf`.
///
/// This will use the default settings for everything, except
/// the root page, which will display a simple "Hello, world!"
/// message
fn main()
{
    ServerBuilder::new()
        .router(FileRouter::new())
        .on_page("/*", |args| {
            let file_path = args["file"].clone();

            let file: PathBuf;
            if file_path.is_empty() {
                file = ".".into();
            }
            else {
                file = args["file"].clone().into();
            }

            if file.is_file() {
                View::path(file_path)
            }
            else if file.is_dir() {
                let mut contents = Vec::new();
                for entry in file.read_dir().unwrap() {
                    if let Ok(entry) = entry {
                        let entry = entry.path();
                        let entry = entry.to_str().unwrap();
                        contents.push(entry.to_owned());
                    }
                }

                View::from(contents.join("\n"))
            }
            else {
                let msg = format!("file \"{}\" does not exist", file_path);
                View::from(msg)
            }
        })
        .start()
        .unwrap();
}

impl FileRouter
{
    fn new() -> Self
    {
        FileRouter {}
    }
}

impl Router for FileRouter
{
    fn resolver(&self, route_spec: String) -> Box<RouteResolver>
    {
        // split the string on all slashes
        let tokens: Vec<String> = route_spec.split("/")
            .skip(1) // skip leading space
            .map(String::from)
            .collect();

        // ensure that no literals follow a star
        {
            let trail = tokens.iter()
                .skip_while(|it| it.as_str() != "*")
                .skip(1) // skip the star element
                .next();

            if let Some(it) = trail {
                panic!("Detected trailing path literal after *: \"{}\"", it)
            }
        }

        Box::new(FileResolver::new(tokens))
    }
}

impl FileResolver
{
    pub fn new(tokens: Vec<String>) -> Self
    {
        FileResolver {
            path: tokens
        }
    }
}

impl RouteResolver for FileResolver
{
    fn resolve(&self, route: &Vec<&str>) -> Option<RouteMap>
    {
        let full_path = route.join("/");
        let mut map = RouteMap::new();
        map.insert("file".to_owned(), full_path);
        let map = map;

        for i in 0..route.len() {
            let actual = route[i];

            match self.path.get(i) {
                None => return None,

                Some(spec) => {
                    let spec = spec.as_str();

                    // if we go to this point, and we see a *, we're going
                    // to return with the full path always
                    if spec == "*" {
                        return Some(map);
                    }
                    // if the spec literal doesn't match what we have, then
                    // we'll reject it
                    else if spec != actual {
                        return None;
                    }
                }
            };
        }

        // if we got here, then that means we matched a fully-literal path
        Some(map)
    }
}
