document.addEventListener('DOMContentLoaded', () => {
    
    const topContainer = document.querySelectorAll('.top-container');

    topContainer.forEach(container => {
        container.classList.add('container-show');
    })
});

async function enrollUser() {
    try {
        const response = await fetch("/enroll-user");

        if (!response.ok) {
            console.error("Failed to enroll user:", response.statusText);
            return;
        }

        window.location.reload();

    } catch (error) {
        console.error("Failed to reset configuration:", error);
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
