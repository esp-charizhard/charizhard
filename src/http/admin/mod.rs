mod config;
mod html;

pub use config::set_routes as set_config_routes;
pub use html::admin_html;

use super::check_ip;
