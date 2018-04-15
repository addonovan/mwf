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
        // this will match any route requested by the server
        .on_page("/*", |args| {
            let file_path = args["file"].clone();

            // convert the requested file into a PathBuf
            // if it's empty, that means we're at the current
            // working directory
            let file: PathBuf;
            if file_path.is_empty() {
                file = ".".into();
            }
            else {
                file = args["file"].clone().into();
            }

            // if it's a file, we'll display the contents of the file
            if file.is_file() {
                View::path(file_path)
            }
            // if it's a directory, we'll list its contents
            else if file.is_dir() {
                let mut contents = Vec::new();
                for entry in file.read_dir().unwrap() {
                    if let Ok(entry) = entry {

                        // sometimes rust is very annoying
                        // Path -> &OsStr -> OsString -> String
                        // If you know of an easier way, please file a PR and
                        // let me know, because this is absurd in my opinion
                        let entry = entry.path();
                        let entry = entry.file_name().unwrap();
                        let entry = entry.to_owned();
                        let entry = entry.into_string().unwrap();

                        contents.push(entry);
                    }
                }

                View::from(contents.join("\n"))
            }
            // if it's neither, idk say it's not real
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
    /// Creates a new FileRouter
    ///
    /// This is just because I prefer `FileRouter::new()` over
    /// `FileRouter {}`.
    fn new() -> Self
    {
        FileRouter {}
    }
}

impl Router for FileRouter
{
    fn resolver(&self, route_spec: String) -> Box<RouteResolver>
    {
        // route specs are guaranteed to never have a leading or trailing /
        let tokens: Vec<String> = route_spec.split("/")
            .map(String::from)
            // treat empty strings as meaning `None`, and remove them
            .filter_map(|it| {
                if it.is_empty() {
                    None
                }
                else {
                    Some(it)
                }
            })
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
    /// Creates a new FileResolver using the given string tokens.
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
