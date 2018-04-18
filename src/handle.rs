use routing::RouteMap;
use view::ViewResult;

/// Handles a request on the server.
///
/// If this handler is invoked, then you may assume that it has been correctly
/// routed to this handler.
pub trait RequestHandler
    where Self: Send + Sync
{
    /// Handles a request whose URL tokens are given in the [route_map].
    fn handle(&self, route_map: RouteMap) -> ViewResult;
}
