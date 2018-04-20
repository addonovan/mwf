extern crate mwf;
extern crate hyper;

use mwf::{ServerBuilder};
use mwf::{View, Result, RequestHandler};
use mwf::{Resolver, RouteMap, ResolveParams};

/// Displays the resolved file. If the file is a directory, then its contents
/// will be displayed in a list of links.
struct Browser;

/// Resolves any path so long as a file with the same name exists in the
/// current working directory.
///
/// When a file is matched its full path (relative to the current working
/// directory) will be inserted into the RouteMap under the key `file`.
struct FileResolver;

fn main()
{
    ServerBuilder::new()
        .resolver(|_, _| Box::new(FileResolver))
        .bind("*", Browser)
        .start();
}

impl RequestHandler for Browser
{
    fn handle(&self, route_map: RouteMap) -> Result<View>
    {
        use std::path::PathBuf;

        // copy the path as a string and the file as a PathBuf out from the
        // file attribute. If the file attribute isn't there, then we know
        // that the resolver claims it didn't exist
        let (path, file): (String, PathBuf) = match route_map.get("file") {
            None => {
                return Ok(View::raw("No such file"))
            },

            Some(it) => (it.clone(), it.into())
        };

        // if it's a file, then it's really easy:
        // just look at its contents
        if file.is_file() {
            return View::file(file);
        }

        // otherwise, it's a directory. So we have to list all of the contents
        // of the directory (with links!)
        let mut contents = Vec::new();
        for entry in file.read_dir().unwrap() {

            // Make sure that the entry was alright
            let entry = match entry {
                Err(_) => continue,
                Ok(x) => x,
            };

            // :(
            // Path -> &OsStr -> Option<OsString> -> OsString
            //      -> Option<String> -> String
            let entry = entry.path();
            let entry = entry.file_name().unwrap();
            let entry = entry.to_owned();
            let entry = entry.into_string().unwrap();

            // generate the link, relative to the root of the web server
            // if we don't check for an empty path, then we could wind up with
            // a leading //, which would screw a lot of stuff up
            let link: String;
            if path.is_empty() {
                link = format!("/{}", entry);
            }
            else {
                link = format!("/{}/{}", path, entry);
            }

            // create the actual link for the entry
            let link = format!("<a href='{}'> {} </a>", link, entry);
            contents.push(link);
        }

        // try to make this part into a decorator!

        // build a valid HTML page from the list of links
        let contents = contents.join("<br/>");
        let contents = format!("<html><body>{}</body></html>", contents);

        // manually create a view that shows the directory listing as html
        let mut view = View::raw(contents);
        view.mime = "text/html".parse().unwrap();

        Ok(view)
    }
}

impl Resolver for FileResolver
{
    fn resolve(&self, params: &ResolveParams) -> Option<RouteMap>
    {
        use hyper::Method;
        use std::path::PathBuf;

        // we'll only respond to Get requests
        if params.method != Method::Get {
            return None;
        }

        // get the route from the URL
        let mut map = RouteMap::new();
        let path: String = match params.route.join("/").as_str() {
            "" => ".".into(),
            x => x.into()
        };

        // does that file exist?
        let file: PathBuf = path.clone().into();
        if file.exists() {
            map.insert("file".into(), path);
        }

        Some(map)
    }
}
