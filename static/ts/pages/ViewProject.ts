const loading_modal: HTMLDialogElement = document.getElementById(
    "loading_modal"
) as HTMLDialogElement;
const loading_modal_inner: HTMLDialogElement = document.getElementById(
    "loading_modal_inner"
) as HTMLDialogElement;

const edit_script_form: HTMLFormElement | null = document.getElementById(
    "save_deployment_script"
) as HTMLFormElement | null;

if (edit_script_form) {
    // create project
    edit_script_form.addEventListener("submit", async (e) => {
        e.preventDefault();
        const res = await fetch(edit_script_form.action, {
            method: "POST",
            body: JSON.stringify({
                script: edit_script_form.script.value,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();

        if (json.success === false) {
            alert(json.message);
        } else {
            (
                document.getElementById(
                    "deployment_script"
                ) as HTMLDialogElement
            ).close();
        }
    });
}

const delete_button: HTMLButtonElement | null = document.getElementById(
    "delete_project"
) as HTMLButtonElement | null;

if (delete_button) {
    // delete project
    delete_button.addEventListener("click", async (e) => {
        e.preventDefault();

        if (
            !confirm("Are you sure you want to do this? It cannot be undone.")
        ) {
            return;
        }

        loading_modal_inner.innerHTML =
            "<b>Deleting container!</b> Please wait.";
        loading_modal.showModal();

        const res = await fetch(delete_button.getAttribute("data-endpoint")!, {
            method: "DELETE",
        });

        loading_modal.close();

        const json = await res.json();

        if (json.success === false) {
            alert(json.message);
        } else {
            window.location.href = "/dashboard/projects";
        }
    });
}

// default export
export default {};
