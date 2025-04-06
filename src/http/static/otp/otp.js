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
			const response = await fetch("/verify-otp", {
				method: "POST",
				headers: {
					"Content-Type": "application/x-www-form-urlencoded",
				},
				body: data,
			});

			if (response.status === 200) {
				errorDiv.textContent = "Please enroll finger.";
				errorDiv.style.color = "green";

				const response = await fetch("/enroll-user");

				if(response.ok) {
					errorDiv.textContent = "OK.";
					window.location.href = "/status";
				}
				else {
					errorDiv.textContent = "Failed to enroll finger.";
				}
				
			} else {
				errorDiv.textContent = "Failed to verify OTP.";
				errorDiv.style.color = "red";
			}
		} catch (err) {
			console.error("Error submitting form:", err);
			errorDiv.textContent = "Error";
			errorDiv.style.color = "red";
		}
	});
});
