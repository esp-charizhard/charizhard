/// Handles static routes (svgs, css, javascript).
mod assets;
/// Handles the main page route.
mod html;
/// Handles wifi related routes.
mod wifi;
/// Handles wireguard related routes.
mod wireguard;

pub use assets::set_routes as set_assets_routes;
pub use html::index_html;
pub use wifi::set_routes as set_wifi_routes;
pub use wireguard::set_routes as set_wg_routes;

use super::check_ip;
