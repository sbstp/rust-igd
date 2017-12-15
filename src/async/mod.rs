mod gateway;
mod search;

pub use self::search::{search_gateway, search_gateway_from, search_gateway_timeout,
                       search_gateway_from_timeout, get_control_url};
pub use self::gateway::Gateway;
