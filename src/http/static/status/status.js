document.addEventListener("DOMContentLoaded", () => {
	const topContainer = document.querySelectorAll(".top-container");

	topContainer.forEach((container) => {
		container.classList.add("container-show");
	});
});

document.addEventListener("DOMContentLoaded", function () {
	function updateStatus() {
		fetchWifiStatus();
		fetchWireguardStatus();
	}

	// Connect to wireguard on page load
	connectWireguard();

	updateStatus();
	setInterval(updateStatus, 2000);
});

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

async function disconnect() {
	try {
		const [wifiResponse, wgResponse] = await Promise.all([
			fetch("/disconnect-wifi"),
			fetch("/disconnect-wg"),
		]);

		if (!wifiResponse.ok) {
			console.error("Failed to disconnect from wifi:", wifiResponse.statusText);
		}

		if (!wgResponse.ok) {
			console.error("Failed to destroy wireguard tunnel:", wgResponse.statusText);
		}

		if (wifiResponse.ok && wgResponse.ok) {
			window.location.href = "/";
		}
	} catch (error) {
		console.error("Failed to disconnect:", error);
	}
}

async function connectWireguard() {
	try {
		const response = await fetch("/connect-wg");

		if (!response.ok) {
			console.error("Failed to establish wireguard tunnel:", response.statusText);
			return;
		}
	} catch (error) {
		console.error("Failed to establish wireguard tunnel:", error);
	}
}
