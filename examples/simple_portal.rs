extern crate mwf;

use mwf::{View, ViewResult, RequestHandler, ServerBuilder};
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;
use mwf::routing::RouteMap;

#[derive(Debug)]
struct NoSuchUserError;

/// A user in our "system"
struct User
{
    /// the users unique* id
    ///
    /// * can be reassigned and conflicts aren't checked then
    id: u32,

    /// The name of the user
    name: String,
}

struct UserPortal
{
    users: Arc<Mutex<Vec<User>>>,
}

impl UserPortal
{
    fn new(users: Vec<User>) -> Self
    {
        UserPortal {
            users: Arc::new(Mutex::new(users)),
        }
    }
}

impl RequestHandler for UserPortal
{
    fn handle(&self, route_map: RouteMap) -> ViewResult
    {
        // grab the deets from the URL
        let id: u32 = route_map[":id"].parse().unwrap();
        let action = route_map[":action"].as_str();
        let arg = route_map.get(":arg?");

        let mut users = self.users.lock().unwrap();

        // grab the user specified by the id
        let user = match users.iter_mut().find(|it| it.id == id) {
            None => return Err(Box::new(NoSuchUserError::new())),
            Some(x) => x,
        };

        // try to perform the given action
        let text: String = match action {
            "greet" => {
                format!("Hello, {}!", user.name)
            },

            "change_id" => {
                match arg.and_then(|it| it.parse::<u32>().ok()) {
                    None => {
                        "Dude, that's not a number".into()
                    },

                    Some(id) => {
                        user.id = id;
                        "Success!".into()
                    }
                }
            },

            "change_name" => {
                match arg {
                    None => {
                        "No name given".into()
                    },

                    Some(name) => {
                        user.name = name.clone();
                        "Success!".into()
                    }
                }
            }

            _ => {
                "Unknown action :(".into()
            }
        };

        View::from(text)
    }
}

fn main()
{
    // generate a list of user information, realistically, this would
    // probably be a databse
    let mut users = Vec::new();
    users.push(User::new(1, "Austin Donovan"));
    users.push(User::new(2, "John Doe"));
    users.push(User::new(3, "Jane Doe"));

    // create a user portal using the users we made
    // we'll wrap it in an arc-mutex so that it's thread-safe
    let portal = UserPortal::new(users);

    ServerBuilder::new()
        .bind("/user/:id/:action/:arg?", portal)
        .start()
        .unwrap();
}


impl User
{
    fn new<T: Into<String>>(id: u32, name: T) -> User
    {
        User {
            id,
            name: name.into(),
        }
    }
}

impl NoSuchUserError
{
    fn new() -> NoSuchUserError
    {
        NoSuchUserError {}
    }
}

impl Error for NoSuchUserError
{
    fn description(&self) -> &str
    {
        "That user don't real"
    }
}

impl fmt::Display for NoSuchUserError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "That user didn't exist :(")
    }
}
