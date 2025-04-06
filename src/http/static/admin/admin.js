document.addEventListener("DOMContentLoaded", () => {
	const topContainer = document.querySelectorAll(".top-container");

	topContainer.forEach((container) => {
		container.classList.add("container-show");
	});
});

async function resetConfig() {
	try {
		const response = await fetch("/reset-config");

		if (!response.ok) {
			console.error("Failed to reset configuration:", response.statusText);
			return;
		}

		document.getElementById('cert').innerHTML = ''
		document.getElementById('certprivkey').innerHTML = ''
		window.location.reload();

	} catch (error) {
		console.error("Failed to reset configuration:", error);
	}
}

async function setConfig() {
	try {
		const response = await fetch("/set-config");

		if (!response.ok) {
			console.error("Failed to reset configuration:", response.statusText);
			return;
		}

		document.getElementById('cert').innerHTML = ''
		document.getElementById('certprivkey').innerHTML = ''
		window.location.reload();

	} catch (error) {
		console.error("Failed to reset configuration:", error);
	}
}
