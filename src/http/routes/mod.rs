mod admin;
mod otp;
mod r#static;
mod wifi;
mod wireguard;

pub use admin::set_routes as set_admin_routes;
pub use otp::set_routes as set_otp_routes;
pub use r#static::set_routes as set_static_routes;
pub use wifi::set_routes as set_wifi_routes;
pub use wireguard::set_routes as set_wg_routes;

use super::check_ip;
