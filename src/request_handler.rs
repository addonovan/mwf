use resolution::RouteMap;
use view::View;
use error::Result;

/// The handler for a request. This is only called if its related [Resolver] has
/// decided it accepts the requested path.
pub trait RequestHandler
    where Self: Send + Sync
{
    /// Handles the request and returns the view to display.
    fn handle(&self, route_map: RouteMap) -> Result<View>;
}
