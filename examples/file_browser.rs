extern crate mwf;

use mwf::{View, ViewResult, ServerBuilder, RequestHandler};
use mwf::routing::{Resolver, RouteMap};

use std::path::PathBuf;

/// This resolves only if the specified file in the URL exists.
struct FileResolver;

struct FileBrowser;
impl RequestHandler for FileBrowser
{
    fn handle(&self, route_map: RouteMap) -> ViewResult
    {
        let file_path = route_map["file"].to_string();

        // convert the requested file into a PathBuf
        // if it's empty, that means we're at the current
        // working directory
        let file: PathBuf;
        if file_path.is_empty() {
            file = ".".into();
        }
        else {
            file = file_path.clone().into();
        }

        // if it's a file, we'll display the contents of the file
        if file.is_file() {
            View::path(file_path)
        }
        // if it's a directory, we'll list its contents
        else if file.is_dir() {
            let mut contents = Vec::new();
            for entry in file.read_dir().unwrap() {

                //
                let entry = match entry {
                    Err(_) => continue,
                    Ok(entry) => entry,
                };

                // sometimes rust is very annoying
                // Path -> &OsStr -> OsString -> String
                // If you know of an easier way, please file a PR and
                // let me know, because this is absurd in my opinion
                let entry = entry.path();
                let entry = entry.file_name().unwrap();
                let entry = entry.to_owned();
                let entry = entry.into_string().unwrap();

                let link: String;
                if file_path.is_empty() {
                    link = format!(
                        "<a href=\"/{0:}\"> {0:} </a>",
                        entry
                    );
                }
                else {
                    link = format!(
                        "<a href=\"/{0:}/{1:}\"> {1:} </a>",
                        file_path,
                        entry
                    );
                }

                contents.push(link);
            }

            // separate entries with a newline
            let contents = contents.join("<br/>");

            // wrap it with html tags
            let contents = format!("<html>{}</html>", contents);

            // mark it as a view with an html mime type
            let mut view: View = contents.into();
            view.mime("text/html".parse().unwrap());

            Ok(view)
        }
        // if it's neither, idk say it's not real
        else {
            let msg = format!("file \"{}\" does not exist", file_path);
            View::from(msg)
        }
    }
}

/// The simplest server you can make with `mwf`.
///
/// This will use the default settings for everything, except
/// the root page, which will display a simple "Hello, world!"
/// message
fn main()
{
    ServerBuilder::new()
        .resolver(FileResolver::new)
        .bind("*", FileBrowser {})
        .start()
        .unwrap();
}

impl FileResolver
{
    /// Creates a new FileResolver using the given string tokens.
    pub fn new(_: Vec<String>) -> Box<Resolver>
    {
        Box::new(FileResolver {})
    }
}

impl Resolver for FileResolver
{
    fn resolve(&self, route: &Vec<&str>) -> Option<RouteMap>
    {
        // join it all into a single string, then check if that file exists
        let full_path = route.join("/");
        let file: PathBuf;
        if full_path.is_empty() {
            file = ".".into();
        }
        else {
            file = full_path.clone().into();
        }

        if file.exists() {
            let mut map = RouteMap::new();
            map.insert("file".into(), full_path);
            Some(map)
        }
        else {
            None
        }
    }
}
