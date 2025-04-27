use anyhow::Error;
use esp_idf_svc::http::server::{EspHttpServer, Method};

/// Sets the static routes for the http server.
pub fn set_routes(http_server: &mut EspHttpServer<'static>) -> anyhow::Result<()> {
    http_server.fn_handler("/status.js", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/status/status.js");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "application/javascript")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/status.css", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/status/status.css");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/css")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/otp.js", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/otp/otp.js");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "application/javascript")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/otp.css", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/otp/otp.css");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/css")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/gen_otp.js", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/gen_otp/gen_otp.js");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "application/javascript")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/gen_otp.css", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/gen_otp/gen_otp.css");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/css")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/admin.js", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/admin/admin.js");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "application/javascript")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/admin.css", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/admin/admin.css");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/css")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/index.js", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/index/index.js");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "application/javascript")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/index.css", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let connection = request.connection();

        let file = include_str!("../static/index/index.css");

        connection.initiate_response(200, Some("OK"), &[("Content-Type", "text/css")])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/spinner.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/spinner.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/signal-1.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/signal-1.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/signal-2.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/signal-2.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/signal-3.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/signal-3.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/signal-4.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/signal-4.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/unlocked.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/unlocked.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/locked.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/locked.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/connected.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/connected.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    http_server.fn_handler("/disconnected.svg", Method::Get, move |mut request| {
        super::check_ip(&mut request)?;

        let file = include_str!("../static/assets/disconnected.svg");

        let connection = request.connection();

        connection.initiate_response(200, Some("OK"), &[
            ("Content-Type", "image/svg+xml"),
            ("Cache-Control", "public, max-age=3600"),
        ])?;

        connection.write(file.as_bytes())?;

        Ok::<(), Error>(())
    })?;

    Ok(())
}
