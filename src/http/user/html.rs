use base64::prelude::BASE64_STANDARD;
use base64::Engine;

/// Stores the data for the http server's favicon as a byte slice to be included
/// in rendering.
const FAVICON_DATA: &[u8] = include_bytes!("../static/assets/favicon.ico");

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
                        <h1>Wireguard</h1>
                        <div id="wg-status-message"></div>
                        <button onclick="connectWg()">Authenticate & Connect</button>
                    </div>
                    
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
