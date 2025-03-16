mod config;
mod html;
mod cert;

pub use config::set_routes as set_config_routes;
pub use html::admin_html;
pub use cert::fetch_config;

use super::check_ip;
