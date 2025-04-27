document.addEventListener("DOMContentLoaded", () => {
	const topContainer = document.querySelectorAll(".top-container");

	topContainer.forEach((container) => {
		container.classList.add("container-show");
	});
});

document.addEventListener("DOMContentLoaded", () => {
	const form = document.getElementById("config");
	const errorDiv = document.getElementById("mtls-error");

	form.addEventListener("submit", async (e) => {
		e.preventDefault();

		const formData = new FormData(form);
		const data = new URLSearchParams(formData).toString();

		try {
			const response = await fetch("/gen-otp", {
				method: "POST",
				headers: {
					"Content-Type": "application/x-www-form-urlencoded",
				},
				body: data,
			});

			if (response.status === 200) {
				errorDiv.textContent = "OK.";
				window.location.href = "/otp";
			} else {
				errorDiv.textContent = "Failed to send request or email may be invalid.";
				errorDiv.style.color = "red";
			}
		} catch (err) {
			console.error("Error submitting form:", err);
			errorDiv.textContent = "Error";
			errorDiv.style.color = "red";
		}
	});
});
