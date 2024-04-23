const error: HTMLElement = document.getElementById("error")!;
const create_form: HTMLFormElement | null = document.getElementById(
    "create-project"
) as HTMLFormElement | null;

if (create_form) {
    // create project
    create_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch("/api/v1/projects", {
            method: "POST",
            body: JSON.stringify({
                name: create_form._name.value,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            window.location.href = `/dashboard/project/${json.payload.name}`;
        }
    });
}

// default export
export default {};
