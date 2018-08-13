mod gateway;
mod search;

pub use self::gateway::Gateway;
pub use self::search::{
    get_control_url, search_gateway, search_gateway_from, search_gateway_from_timeout,
    search_gateway_timeout,
};
