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
            "<b>Releasing resources!</b> Please wait.";
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

// file management
for (const element of Array.from(
    document.querySelectorAll(".load_file_info")
)) {
    element.addEventListener("click", () => {
        const endpoint = element.getAttribute("data-file-endpoint")!;
        const project = element.getAttribute("data-project")!;
        const file = element.getAttribute("data-file")!;

        (globalThis as any).delete_file = async () => {
            if (
                !confirm(
                    "Are you sure you want to do this? It cannot be undone."
                )
            ) {
                return;
            }

            loading_modal_inner.innerHTML =
                "<b>Releasing resources!</b> Please wait.";
            loading_modal.showModal();

            const res = await fetch(endpoint, {
                method: "DELETE",
            });

            loading_modal.close();

            const json = await res.json();

            if (json.success === false) {
                alert(json.message);
            } else {
                window.location.reload();
            }
        };

        (globalThis as any).open_file_editor = async () => {
            window.open(
                `${window.location.origin}/dashboard/project/${project}/edit${file}`
            );
        };
    });
}

const upload_button: HTMLButtonElement | null = document.getElementById(
    "upload_file"
) as HTMLButtonElement | null;

const create_button: HTMLButtonElement | null = document.getElementById(
    "create_file"
) as HTMLButtonElement | null;

function base64_file(input: File): Promise<String> {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();

        reader.addEventListener("loadend", () => {
            resolve(
                (reader.result as String)
                    .replace("data:", "")
                    .replace(/^.+,/, "")
            );
        });

        reader.readAsDataURL(input);
    });
}

if (upload_button) {
    // upload file
    upload_button.addEventListener("click", async (e) => {
        e.preventDefault();

        // get file
        const file_input = document.createElement("input");
        file_input.type = "file";
        file_input.click();

        file_input.addEventListener("change", async () => {
            const file_base64 = await base64_file(file_input.files![0]);

            // get path
            const file_path = prompt("File path:");
            if (!file_path) return;

            file_input.remove();

            // ...
            loading_modal_inner.innerHTML =
                "<b>Uploading file!</b> Please wait.";
            loading_modal.showModal();

            const res = await fetch(
                `${upload_button.getAttribute("data-endpoint")!}/${file_path}`,
                {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        content: file_base64,
                    }),
                }
            );

            loading_modal.close();

            const json = await res.json();

            if (json.success === false) {
                alert(json.message);
            } else {
                window.location.reload();
            }
        });
    });
}

if (create_button && upload_button) {
    // create file
    create_button.addEventListener("click", async (e) => {
        e.preventDefault();

        // get path
        let file_path = prompt("File path:");
        if (!file_path) return;

        if (file_path === "/Index.html") {
            // mobile autocap
            file_path = "/index.html";
        }

        const file_base64 = btoa("New File");

        // ...
        loading_modal_inner.innerHTML = "<b>Uploading file!</b> Please wait.";
        loading_modal.showModal();

        const res = await fetch(
            `${upload_button.getAttribute("data-endpoint")!}/${file_path}`,
            {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    content: file_base64,
                }),
            }
        );

        loading_modal.close();

        const json = await res.json();

        if (json.success === false) {
            alert(json.message);
        } else {
            window.location.reload();
        }
    });
}

// live url
const live_url = document.getElementById(
    "live_url"
) as HTMLAnchorElement | null;

if (live_url) {
    // live_url.href = `${window.location.protocol}//${live_url.getAttribute(
    //     "data-project"
    // )!}.get.${window.location.host}`;
    live_url.href = `/${live_url.getAttribute("data-project")!}`;
    live_url.innerText = live_url.href;
}

// move file
for (const element of Array.from(
    document.querySelectorAll(".load_file_path")
)) {
    element.addEventListener("click", () => {
        const endpoint = element.getAttribute("data-file-endpoint")!;

        (globalThis as any).move_file = async (e: any) => {
            e.preventDefault();

            loading_modal_inner.innerHTML =
                "<b>Moving resources!</b> Please wait.";
            loading_modal.showModal();

            const res = await fetch(endpoint, {
                method: "POST",
                body: JSON.stringify({
                    path: e.target.new_file_path.value,
                }),
                headers: {
                    "Content-Type": "application/json",
                },
            });

            loading_modal.close();

            const json = await res.json();

            if (json.success === false) {
                alert(json.message);
            } else {
                window.location.reload();
            }
        };
    });
}

// default export
export default {};
