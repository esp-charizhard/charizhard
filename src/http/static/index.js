document.addEventListener('DOMContentLoaded', () => {
    
    const topContainer = document.querySelectorAll('.top-container');
    const statusContainer = document.querySelectorAll('.center-container')

    topContainer.forEach(container => {
        container.classList.add('container-show');
    })
    statusContainer.forEach(container => {
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

async function sleep(time) {
    await new Promise(resolve => setTimeout(resolve, time))
}

document.getElementById('config').addEventListener('submit', function(event) {
    let isValid = true;

    function setError(id, message) {
        const errorDiv = document.getElementById(id);
        errorDiv.textContent = message;
        isValid = false;
    }

    function clearError(id) {
        const errorDiv = document.getElementById(id);
        errorDiv.textContent = '';
    }

    // Validate WireGuard Address
    const address = document.getElementById('address').value;
    if (address && !/^(\d{1,3}\.){3}\d{1,3}$/.test(address)) {
        setError('address-error', "Must be a valid IP address.");
    } else {
        clearError('address-error');
    }

    // Validate WireGuard Port
    const port = document.getElementById('port').value;
    if (port && (!/^\d{1,5}$/.test(port) || parseInt(port) > 65535)) {
        setError('port-error', "Must be a valid port between 0 and 65535.");
    } else {
        clearError('port-error');
    }

    // Validate Client Private Key
    const privKey = document.getElementById('privkey').value;
    if (privKey.length != 44) {
        setError('privkey-error', "Must be 44 characters long.");
    } else {
        clearError('privkey-error');
    }

    // Validate Remote Host Public Key
    const pubKey = document.getElementById('pubkey').value;
    if (pubKey.length != 44) {
        setError('pubkey-error', "Must be 44 characters long.");
    } else {
        clearError('pubkey-error');
    }

    if (!isValid) {
        event.preventDefault();
    }
});

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

async function resetConfig() {
    try {
        const response = await fetch("/reset-config");

        if (!response.ok) {
            console.error("Failed to reset configuration:", response.statusText);
            return;
        }

        window.location.reload();

    } catch (error) {
        console.error("Failed to reset configuration:", error);
    }
}

async function fetchConfig() {
    try {
        const response = await fetch('/fetch-config');
        if (!response.ok) {
            console.error("Failed to fetch config:", response.statusText);
            return
        }

        const text = await response.text();
        const params = new URLSearchParams(text);

        // Fill the input fields with the received data
        document.getElementById('address').value = params.get('address') || '';
        document.getElementById('port').value = params.get('port') || '';
        document.getElementById('privkey').value = params.get('privkey') || '';
        document.getElementById('pubkey').value = params.get('pubkey') || '';

        // Handle the "remember me" checkbox
        document.getElementById('remember-me').checked = params.get('rember') === 'true';

    } catch (error) {
        console.error('Error fetching config:', error);
    }
}