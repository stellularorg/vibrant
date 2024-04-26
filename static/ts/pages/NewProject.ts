const error: HTMLElement = document.getElementById("error")!;

const loading_modal: HTMLDialogElement = document.getElementById(
    "loading_modal"
) as HTMLDialogElement;
const loading_modal_inner: HTMLDialogElement = document.getElementById(
    "loading_modal_inner"
) as HTMLDialogElement;

const create_form: HTMLFormElement | null = document.getElementById(
    "create-project"
) as HTMLFormElement | null;

if (create_form) {
    // create project
    create_form.addEventListener("submit", async (e) => {
        e.preventDefault();

        loading_modal_inner.innerHTML =
            "<b>Provisioning resources!</b> Please wait.";
        loading_modal.showModal();

        const project_type = create_form.project_type;
        const res = await fetch("/api/v1/projects", {
            method: "POST",
            body: JSON.stringify({
                name: create_form._name.value,
                type: (
                    project_type.options[
                        project_type.selectedIndex
                    ] as HTMLOptionElement
                ).value,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        loading_modal.close();

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
