document.addEventListener("DOMContentLoaded", () => {
	const topContainer = document.querySelectorAll(".top-container");

	topContainer.forEach((container) => {
		container.classList.add("container-show");
	});

	const cookies = document.cookie.split("; ").reduce((acc, cookie) => {
		const [name, value] = cookie.split("=");
		acc[name] = decodeURIComponent(value);
		return acc;
	}, {});

	if (cookies.email) {
		const emailInput = document.getElementById("email");
		emailInput.value = cookies.email;
	}
});

document.addEventListener("DOMContentLoaded", () => {
    const form = document.getElementById("config");
    const errorDiv = document.getElementById("mtls-error");
    const loadingSvg = document.getElementById("loading-svg");
    const submitButton = document.getElementById("submit-button");
    const otpInput = document.getElementById("otp");
    const otpLabel = document.querySelector('label[for="otp"]');
	let redirecting = false;

    form.addEventListener("submit", async (e) => {
        e.preventDefault();

        const formData = new FormData(form);
        const data = new URLSearchParams(formData).toString();

        try {
            loadingSvg.style.display = "flex";
            submitButton.disabled = true;
            otpInput.style.display = "none";
            otpLabel.style.display = "none";

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

                const enrollResponse = await fetch("/enroll-user");

                if (enrollResponse.ok) {
                    redirecting = true;
					errorDiv.textContent = "OK.";
                    window.location.href = "/status";

                } else {
                    errorDiv.textContent = "Failed to enroll finger.";
                    errorDiv.style.color = "red";
                }
            } else {
                errorDiv.textContent = "Failed to verify OTP.";
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
            	otpInput.style.display = "inline-block";
            	otpLabel.style.display = "inline-block";
			}
		}
    });
});
