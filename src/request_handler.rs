use resolution::RouteMap;

pub trait RequestHandler
    where Self: Send + Sync
{
    fn handle(&self, route_map: RouteMap) -> String;
}
