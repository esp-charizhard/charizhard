document.addEventListener("DOMContentLoaded", () => {
	const topContainer = document.querySelectorAll(".top-container");

	topContainer.forEach((container) => {
		container.classList.add("container-show");
	});
});

document.addEventListener("DOMContentLoaded", () => {
	const form = document.getElementById("config");
	const errorDiv = document.getElementById("mtls-error");
	const loadingSvg = document.getElementById("loading-svg");
	const submitButton = document.getElementById("submit-button");
	const emailInput = document.getElementById("email");
	const emailLabel = document.querySelector('label[for="email"]');
	let redirecting = false;

	form.addEventListener("submit", async (e) => {
		e.preventDefault();

		const formData = new FormData(form);
		const data = new URLSearchParams(formData).toString();

		try {
			loadingSvg.style.display = "flex";
			submitButton.disabled = true;
			emailInput.style.display = "none";
			emailLabel.style.display = "none";
			errorDiv.textContent = "";

			const response = await fetch("/gen-otp", {
				method: "POST",
				headers: {
					"Content-Type": "application/x-www-form-urlencoded",
				},
				body: data,
			});

			if (response.status === 200) {
				const email = formData.get("email");
				document.cookie = `email=${encodeURIComponent(
					email
				)}; path=/; max-age=3600; SameSite=Strict`;

				redirecting = true;
				window.location.href = "/otp";
			} else {
				errorDiv.textContent = "Failed to send request or email may be invalid.";
				errorDiv.style.color = "red";
			}
		} catch (err) {
			console.error("Error submitting form:", err);
			errorDiv.textContent = "Error";
			errorDiv.style.color = "red";
		} finally {
			if (!redirecting) {
				loadingSvg.style.display = "none";
				submitButton.disabled = false;
				emailInput.style.display = "inline-block";
				emailLabel.style.display = "inline-block";
			}
		}
	});
});
