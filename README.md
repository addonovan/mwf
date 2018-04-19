mwf
===

General
---

*M*y *W*eb *F*ramework. (Great name, I know)

This project was inspired by Gary Berndhart's screencast on 
[Destroy All Software](https://www.destroyallsoftware.com/) about making a
webframework from scratch.

This is not intended to be used in a production or production-like setting, so
please don't even try to do so.

About
---

This is a backend web framework, like ASP.NET. The most important part of this
is the router and route resolvers, which take in a route that has been requested
from the server and try to find the handler which should accept it. 

Examples
---

Examples can be found in the [examples](examples) directory.

Here is a simple "Hello World" server:
```rust
struct HelloWorld;
impl RequestHandler for HelloWorld
{
    fn handle(&self, _args: RouteMap) -> ViewResult
    {
        View::from("Hello, world!")
    }
}

fn main()
{
    ServerBuilder::new()
        .bind("/", HelloWorld {})
        .start()
        .unwrap();
}
```

Routing
---

The `Resolver` trait, whose definition is listed below, will simply take in a
URL which has been split up along the slashes, and return `None` if the path
isn't accepted by the resolver, or `Some(RouteMap)` if the route was accepted.
A `RouteMap` is simply `HashMap<String, String>`, which is filled with any
information the resolver might want to pass along to the handler.
```rust
pub trait Resolver
    where Self: Send + Sync
{
    fn resolve(&self, route: &Vec<&str>) -> Option<RouteMap>;
}
```
The standard resolver (which is the default one enabled) has three path tokens:
* a literal token (`/foo`)
    * This matches text exactly as it appears in a URL
* a variable token (`/:foo`)
    * This matches any text in the same position, and the text it matched will
      be stored in the `RouteMap` under the variable's name
    * Variables must begin with a leading `:` which *is* used in its name in the
      `RouteMap`.
* an optional variable token (`/:foo?`)
    * This matches anything in the same position (including nothing).
    * The matched text will be stored in the `RouteMap` under the variable's
      name (`""` if it matched nothing)
    * Like variables, optional variables must begin with a `:` and must also end
      with a `?`. Both of these characters are part of the variable's name in
      the `RouteMap`.
      
Using these tokens, you can build route handlers for a lot of things.

Request Handlers
---

On the other side of request resolution, is the actual handler itself. Handler
is also a trait (shown below), but a much simpler one. This receives the
`RouteMap` generated by its corresponding `Resolver` and then serves a `View`
of the content which is meant to be at the URL. Because generating the page
might cause an error (who hasn't seen a `500 Internal Service Error` before?),
we don't want the entire server to crash on such a problem. For this reason, the
handler must actually return a `ViewResult` (aka `Result<View, Box<Error>>`).
```rust
pub trait RequestHandler
    where Self: Send + Sync
{
    fn handle(&self, route_map: RouteMap) -> ViewResult;
}
```

Using the `ServerBuilder` interface, handlers are attached to the server by
using the `ServerBuilder.bind(self, route_spec, handler)` method, as shown above
in the hello world example.

Dependencies
---

It uses  [iron](https://github.com/iron/iron/) as the underlying HTTP server, 
because I don't actually ~~hate myself~~ want to implement an HTTP server, as
 well as [pulldown-cmark](https://github.com/google/pulldown-cmark) for 
converting markdown to HTML.

Of course, this is rust and you don't actually need to know that, but I thought
I would mention it anyways.
