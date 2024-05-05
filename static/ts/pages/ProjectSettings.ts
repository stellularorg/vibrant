const error: HTMLElement = document.getElementById("error")!;

const loading_modal: HTMLDialogElement = document.getElementById(
    "loading_modal"
) as HTMLDialogElement;
const loading_modal_inner: HTMLDialogElement = document.getElementById(
    "loading_modal_inner"
) as HTMLDialogElement;

const edit_fields_form: HTMLFormElement | null = document.getElementById(
    "update_project_information"
) as HTMLFormElement | null;

if (edit_fields_form) {
    // edit project info
    edit_fields_form.addEventListener("submit", async (e) => {
        e.preventDefault();

        loading_modal_inner.innerHTML =
            "<b>Updating resources!</b> Please wait.";
        loading_modal.showModal();

        const res = await fetch(
            edit_fields_form.getAttribute("data-endpoint")!,
            {
                method: "POST",
                body: JSON.stringify({
                    name: edit_fields_form._name.value,
                    owner: edit_fields_form.owner.value,
                }),
                headers: {
                    "Content-Type": "application/json",
                },
            }
        );

        loading_modal.close();

        const json = await res.json();

        if (json.success === false) {
            error.style.display = "block";
            error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
        } else {
            window.location.href = `/dashboard/project/${json.payload}/settings`;
        }
    });
}

// default export
export default {};
