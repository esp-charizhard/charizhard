use base64::prelude::BASE64_STANDARD;
use base64::Engine;

/// Stores the data for the http server's favicon as a byte slice to be included
/// in rendering.
const FAVICON_DATA: &[u8] = include_bytes!("../static/assets/favicon.ico");

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
                        <button onclick="enrollUser()">Enroll User</button>
                        <button onclick="resetConfig()">Factory Reset</button>
                        
                    </div>
                </body>
                <script src="admin.js"></script>
            </html>
        "###
    ))
}
