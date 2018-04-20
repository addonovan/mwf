use resolution::RouteMap;
use view::View;
use error::Result;

pub trait RequestHandler
    where Self: Send + Sync
{
    fn handle(&self, route_map: RouteMap) -> Result<View>;
}
