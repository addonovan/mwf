extern crate mwf;

use mwf::{View, ServerBuilder};
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct NoSuchUserError;

/// A user portal, i.e. a list of users
struct UserPortal
{
    users: Vec<User>,
}

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
    let portal = Arc::new(Mutex::new(UserPortal::new(users)));

    // we have to make clones of portal before we start, so that the clones
    // can each be moved safely into the closure and owned by them
    let context1 = portal.clone();
    let context2 = portal.clone();
    let context3 = portal.clone();
    let context4 = portal.clone();

    ServerBuilder::new()
        // if we're just supplied the user's id, we'll just call the view_user
        // method on the UserPortal (everything else is just boilerplate and
        // error handling)
        .on_page("/user/:id/", move |args| {
            let id: u32 = args[":id"].parse()?;

            // if the text couldn't be generated, then the user
            // with that id didn't exist
            let text = match context1.lock().unwrap().view_user(id) {
                None => return Err(Box::new(NoSuchUserError::new())),
                Some(x) => x,
            };

            View::from(text)
        })
        // if we're asked to greet the user, we'll say hi to them
        .on_page("/user/:id/greet", move |args| {
            let id: u32 = args[":id"].parse()?;
            let portal = context2.lock().unwrap();

            // make sure that the user exists
            let user = match portal.user(id) {
                None => return Err(Box::new(NoSuchUserError::new())),
                Some(x) => x,
            };

            let text = format!("Hello, {} (id = {})", user.name, user.id);

            View::from(text)
        })
        // if we're supposed to edit the id of the user, we also need the
        // new id. Then, we'll change the user's id
        .on_page("/user/:id/edit/id/:new_id", move |args| {
            let id: u32 = args[":id"].parse()?;
            let new_id: u32 = args[":new_id"].parse()?;

            let mut portal = context3.lock().unwrap();

            // make sure that the user exists
            let user = match portal.user_mut(id) {
                None => return Err(Box::new(NoSuchUserError::new())),
                Some(x) => x,
            };

            // realistically, we should check if another user has that
            // same new_id, but oh well, this is just an example of the
            // software, not an actual service
            user.id = new_id;

            View::from("Success!")
        })
        // same as above, but for the user's name
        .on_page("/user/:id/edit/name/:new_name", move |args| {
            let id: u32 = args[":id"].parse()?;
            let new_name = args[":new_name"].clone();

            let mut portal = context4.lock().unwrap();

            // make sure that the user exists
            let user = match portal.user_mut(id) {
                None => return Err(Box::new(NoSuchUserError::new())),
                Some(x) => x,
            };

            // realistically, we should check if another user has that
            // same new_id, but oh well, this is just an example of the
            // software, not an actual service
            user.name = new_name;

            View::from("Success!")
        })
        .start()
        .unwrap();
}

impl UserPortal
{
    fn new(users: Vec<User>) -> UserPortal
    {
        UserPortal {
            users
        }
    }

    fn user(&self, id: u32) -> Option<&User>
    {
        self.users.iter()
            .find(|it| it.id == id)
    }

    fn user_mut(&mut self, id: u32) -> Option<&mut User>
    {
        self.users.iter_mut()
            .find(|it| it.id == id)
    }

    fn view_user(&self, id: u32) -> Option<String>
    {
        self.user(id)
            .and_then(|user| {
                Some(format!("Your name is, {}", user.name))
            })
    }
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
