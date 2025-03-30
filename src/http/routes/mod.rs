mod wifi;
mod wireguard;
mod r#static;
mod admin;

pub use wifi::set_routes as set_wifi_routes;
pub use wireguard::set_routes as set_wg_routes;
pub use r#static::set_routes as set_static_routes;
pub use admin::set_routes as set_admin_routes;

use super::check_ip;