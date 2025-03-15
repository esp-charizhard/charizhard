document.addEventListener('DOMContentLoaded', () => {
    const topContainer = document.querySelectorAll('.top-container');

    topContainer.forEach(container => {
        container.classList.add('container-show');
    })
});

document.addEventListener("DOMContentLoaded", function() {
    fetchWifiStatus();
    fetchWireguardStatus();

    setInterval(fetchWifiStatus, 2500);
    setInterval(fetchWireguardStatus, 2500);
});

document.addEventListener("DOMContentLoaded", function() {
    fetchConfig();
});

async function connectWg() {
    const statusDiv = document.getElementById('wg-status-message');

    try {
        const response = await fetch("/connect-wg")

        if (response.status === 401) {
            statusDiv.textContent = "Finger KO";
            statusDiv.style.color = 'red';
        } else if (response.ok) {
            statusDiv.textContent = "Finger OK";
            statusDiv.style.color = 'green';
        } else {
            statusDiv.textContent = "Failed to connect";
            statusDiv.style.color = 'orange';
        }

    } catch (error) {
        console.error("Failed to connect to WireGuard:", error);
        statusDiv.textContent = "Failed to connect";
        statusDiv.style.color = 'orange';
    }
}

function connectWifi(event) {
    event.preventDefault();

    const form = event.target.closest('form');
    const passwordInput = form.querySelector('input[type="password"]');

    if (!passwordInput) {
        form.submit();
        return;
    }

    const wifiContainer = form.closest('.wifi');
    const errorDiv = wifiContainer.querySelector('.error');

    if (passwordInput != null && passwordInput.value.length > 64) {
        errorDiv.textContent = "Password must be 64 characters or less.";
        return;
    }

    errorDiv.textContent = "";

    form.submit();
}

async function fetchScannedWifis() {

    let scanned_wifis = document.getElementById('inner-scanned-wifis');
    scanned_wifis.innerHTML = "";

    try {
        document.getElementById('loading-svg').style.display = 'flex'; 
        
        const response = await fetch('/scan-wifi');
        
        if (!response.ok) throw new Error('Error fetching scanned Wi-Fis.');

        const scannedWifis = await response.text();

        document.getElementById('loading-svg').style.display = 'none';

        scanned_wifis.innerHTML = scannedWifis;

        document.querySelectorAll('.wifi-connect button[type="submit"]').forEach(button => {
            button.addEventListener('click', connectWifi);
        });
    } 
    catch (error) {
        scanned_wifis.style.fontWeight = 'bold';
        scanned_wifis.innerHTML = 'Failed to scan WI-Fis.';
        
        document.getElementById('loading-svg').style.display = 'none';
    }
}

function toggleDropdown(event, element) {
    if (event.target.closest('.wifi-connect')) return;

    const form = element.querySelector('.wifi-connect');
    const wifiContainer = element.closest('.wifi');

    form.classList.toggle('visible');
    wifiContainer.classList.toggle('expanded');
}

async function fetchWireguardStatus() {
    try {
        const response = await fetch("/wg-status");

        if (!response.ok) {
            console.error("Failed to fetch Wireguard status:", response.statusText);
            return;
        }

        const htmlContent = await response.text();
        const statusElement = document.getElementById("wireguard-status");

        if (statusElement.innerHTML.trim() !== htmlContent.trim()) {
            requestAnimationFrame(() => {
                statusElement.innerHTML = htmlContent;
            });
        }

    } catch (error) {
        console.error("Error fetching Wireguard status:", error);
    }
}


async function fetchWifiStatus() {
    try {
        const response = await fetch("/wifi-status");

        if (!response.ok) {
            console.error("Failed to fetch Wi-Fi status:", response.statusText);
            return;
        }

        const htmlContent = await response.text();
        const statusElement = document.getElementById("wifi-status");

        if (statusElement.innerHTML.trim() !== htmlContent.trim()) {
            requestAnimationFrame(() => {
                statusElement.innerHTML = htmlContent;
            });
        }

    } catch (error) {
        console.error("Error fetching Wi-Fi status:", error);
    }
}


async function disconnectWifi() {
    try {
        const response = await fetch("/disconnect-wifi");

        if (!response.ok) {
            console.error("Failed to disconnect from wifi:", response.statusText);
            return;
        }

    } catch (error) {
        console.error("Failed to disconnect from wifi:", error);
    }
}

async function disconnectWg() {
    try {
        const response = await fetch("/disconnect-wg");

        if (!response.ok) {
            console.error("Failed to disconnect from wireguard:", response.statusText);
            return;
        }

    } catch (error) {
        console.error("Failed to disconnect from wireguard:", error);
    }
}