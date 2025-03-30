use base64::prelude::BASE64_STANDARD;
use base64::Engine;

/// Stores the data for the http server's favicon as a byte slice to be included
/// in rendering.
const FAVICON_DATA: &[u8] = include_bytes!("./static/assets/favicon.ico");

/// Gives the html for the "/" handler.
pub fn index_html() -> anyhow::Result<String> {
    let favicon = BASE64_STANDARD.encode(FAVICON_DATA);

    Ok(format!(
        r###"
            <!DOCTYPE html>
            <html lang="en">
                
                <head>
                    <link rel="icon" type="image/png" href="data:image/png;base64,{favicon}">
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Charizhard</title>
                    <link rel="stylesheet" href="index.css">
                </head>
                
                <body>                    
                    <div class="top-container">
                        <h1>Wi-Fi</h1>

                        <div id="scanned-wifis" class="scrollable-box">
                            <div id="inner-scanned-wifis"></div>
                            <img id="loading-svg" src="spinner.svg" alt="Loading...">
                        </div>

                        <button onclick="fetchScannedWifis()">Scan</button>
                    </div>

                </body>
                <script src="index.js"></script>
            </html>
        "###
    ))
}

/// Gives the html for the "/status" handler
pub fn status_html() -> anyhow::Result<String> {
    let favicon = BASE64_STANDARD.encode(FAVICON_DATA);

    Ok(format!(
        r###"
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <link rel="icon" type="image/png" href="data:image/png;base64,{favicon}">
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Charizhard</title>
                    <link rel="stylesheet" href="status.css">
                </head>

                <body>
                    <div class="top-container">
                        <h1>Status</h1>
        
                        <div class="wireguard-status-block">
                            <div class="subtitle">Wireguard</div>
                            <div class="status" id="wireguard-status"></div>
                        </div>
                            
                        <div class="wifi-status-block">
                            <div class="subtitle">Wi-Fi</div>
                            <div class="status" id="wifi-status"></div>
                        </div>
                    </div>
                </body>
                <script src="status.js"></script>
            </html>
        "###
    ))
}

/// Gives the html for the "/otp" handler
pub fn otp_html() -> anyhow::Result<String> {
    let favicon = BASE64_STANDARD.encode(FAVICON_DATA);

    Ok(format!(
        r###"
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <link rel="icon" type="image/png" href="data:image/png;base64,{favicon}">
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Charizhard</title>
                    <link rel="stylesheet" href="otp.css">
                </head>

                <body>
                    <div class="top-container">
                        <h1>Configuration</h1>
                        
                        <form id="config" method="post" action="/verify-otp">
                            <label for="email">Email</label>
                            <input type="text" id="email" name="email" value="" required>
                            <div class="error" id="email-error"></div>

                            <label for="otp">One Time Password</label>
                            <input type="text" id="otp" name="otp" value="" required>
                            <div class="error" id="otp-error"></div>

                            <button type="submit">Verify</button>
                        </form>
                        <div class="enroll-user">
                            <button onclick="enrollUser()">Enroll User</button>
                        </div>
                    </div>
                </body>
                <script src="otp.js"></script>
            </html>
        "###
    ))
}

/// Gives the html for the "/admin" handler
pub fn admin_html() -> anyhow::Result<String> {
    let favicon = BASE64_STANDARD.encode(FAVICON_DATA);

    Ok(format!(
        r###"
            <!DOCTYPE html>
            <html lang="en">
                
                <head>
                    <link rel="icon" type="image/png" href="data:image/png;base64,{favicon}">
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Charizhard</title>
                    <link rel="stylesheet" href="admin.css">
                </head>
                
                <body>
                    <div class="top-container">
                        <h1>Configuration</h1>
                        
                        <form id="config" method="post" action="/set-cert">
                            <label for="cert">Certificate</label>
                            <input type="text" id="cert" name="cert" value="" required>
                            <div class="error" id="cert-error"></div>

                            <label for="privkey">Private Key</label>
                            <input type="text" id="privkey" name="cert" value="" required>
                            <div class="error" id="privkey-error"></div>

                            <button type="submit">Save Config</button>
                        </form>
                        <button onclick="resetConfig()">Factory Reset</button>
                        
                    </div>
                </body>
                <script src="admin.js"></script>
            </html>
        "###
    ))
}
