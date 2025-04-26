document.addEventListener("DOMContentLoaded", () => {
	const topContainer = document.querySelectorAll(".top-container");

	topContainer.forEach((container) => {
		container.classList.add("container-show");
	});
});

async function resetConfig() {
	const errorDiv = document.getElementById("mtls-error");
	errorDiv.textContent = "";

	try {
		const response = await fetch("/reset-config");

		if (!response.ok) {
			errorDiv.textContent = "Failed to reset.";
			errorDiv.style.color = "red";
			return;
		}

		document.getElementById("cert").value = "";
		document.getElementById("certprivkey").value = "";

		errorDiv.textContent = "Success. Dongle will reboot to finish..";
		errorDiv.style.color = "green";
	} catch (error) {
		console.error("Failed to reset configuration:", error);
	}
}

document.addEventListener("DOMContentLoaded", () => {
	const form = document.getElementById("config");
	const errorDiv = document.getElementById("mtls-error");
	errorDiv.textContent = "";

	form.addEventListener("submit", async (e) => {
		e.preventDefault();

		// Get form data using FormData
		const formData = new FormData(form);
		const data = new URLSearchParams(formData).toString();

		try {
			const response = await fetch("/set-config", {
				method: "POST",
				headers: {
					"Content-Type": "application/x-www-form-urlencoded",
				},
				body: data,
			});

			if (response.ok) {
				// Clear the textareas after successful submission
				document.getElementById("cert").value = "";
				document.getElementById("certprivkey").value = "";

				// Optionally, display a success message
				errorDiv.textContent = "Configuration saved successfully.";
				errorDiv.style.color = "green";
			} else {
				errorDiv.textContent = "Failed to save configuration.";
				errorDiv.style.color = "red";
			}
		} catch (err) {
			console.error("Error submitting form:", err);
			errorDiv.textContent = "Error submitting form.";
			errorDiv.style.color = "red";
		}
	});
});
