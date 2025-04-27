document.addEventListener("DOMContentLoaded", () => {
	const topContainer = document.querySelectorAll(".top-container");

	topContainer.forEach((container) => {
		container.classList.add("container-show");
	});
});

async function connectWifi(event) {
	event.preventDefault();

	const form = event.target.closest("form");
	const passwordInput = form.querySelector('input[type="password"]');

	const wifiContainer = form.closest(".wifi");
	const errorDiv = wifiContainer.querySelector(".error");

	errorDiv.textContent = "";

	if (passwordInput != null && passwordInput.value.length > 64) {
		errorDiv.textContent = "Password must be 64 characters or less.";
		return;
	}

	const formData = new FormData(form);
	const data = new URLSearchParams(formData).toString();

	try {
		const response = await fetch("/connect-wifi", {
			method: "POST",
			headers: {
				"Content-Type": "application/x-www-form-urlencoded",
			},
			body: data,
		});

		if (response.ok) {
			let first_boot = await isFirstBoot();

			if (first_boot) {
				window.location.href = "/gen-otp";
			} else {
				window.location.href = "/status";
			}
		} else {
			errorDiv.textContent = "Invalid password.";
			errorDiv.style.color = "red";
		}
	} catch {
		console.error("Error connecting to wifi:", err);
		errorDiv.textContent = "Error connecting to wifi.";
		errorDiv.style.color = "red";
	}
}

async function isFirstBoot() {
	try {
		const response = await fetch("/is-first-boot");

		if (response.status === 204) {
			return true;
		} else {
			return false;
		}
	} catch (error) {
		console.error("Error during fetch:", error);
		return false;
	}
}

async function fetchScannedWifis() {
	let scanned_wifis = document.getElementById("inner-scanned-wifis");
	scanned_wifis.innerHTML = "";

	try {
		document.getElementById("loading-svg").style.display = "flex";

		const response = await fetch("/scan-wifi");

		if (!response.ok) throw new Error("Error fetching scanned Wi-Fis.");

		const scannedWifis = await response.text();

		document.getElementById("loading-svg").style.display = "none";

		scanned_wifis.innerHTML = scannedWifis;

		document.querySelectorAll('.wifi-connect button[type="submit"]').forEach((button) => {
			button.addEventListener("click", connectWifi);
		});
	} catch (error) {
		scanned_wifis.style.fontWeight = "bold";
		scanned_wifis.innerHTML = "Failed to scan WI-Fis.";

		document.getElementById("loading-svg").style.display = "none";
	}
}

function toggleDropdown(event, element) {
	if (event.target.closest(".wifi-connect")) return;

	const form = element.querySelector(".wifi-connect");
	const wifiContainer = element.closest(".wifi");

	form.classList.toggle("visible");
	wifiContainer.classList.toggle("expanded");
}
