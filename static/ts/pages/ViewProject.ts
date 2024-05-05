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
let file_editor_open: boolean = false;
for (const element of Array.from(
    document.querySelectorAll(".load_file_info")
)) {
    element.addEventListener("click", () => {
        const endpoint = element.getAttribute("data-file-endpoint")!;
        const mv_endpoint = element.getAttribute("data-file-mv-endpoint")!;
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

        (globalThis as any).move_file = async (e: any) => {
            e.preventDefault();

            loading_modal_inner.innerHTML =
                "<b>Moving resources!</b> Please wait.";
            loading_modal.showModal();

            const res = await fetch(mv_endpoint, {
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

        // editor
        const doc_title = document.title;

        (globalThis as any).open_file_editor = async () => {
            if (file_editor_open === true) {
                return alert("The editor is already open.");
            }

            file_editor_open = true;

            const iframe = document.createElement("iframe");
            iframe.id = "editor";
            iframe.style.display = "block";
            iframe.style.position = "absolute";
            iframe.style.top = "0";
            iframe.style.left = "0";
            iframe.style.width = "100dvw";
            iframe.style.height = "100dvh";
            iframe.setAttribute("frameborder", "0");

            document.body.appendChild(iframe);

            loading_modal_inner.innerHTML =
                "<b>Loading editor!</b> Please wait.";
            loading_modal.showModal();

            // close modal
            (
                document.getElementById("manage_file") as HTMLDialogElement
            ).close();

            // load editor
            const editor_src = `${window.location.origin}/dashboard/project/${project}/edit${file}`;
            iframe.src = editor_src;

            // show editor
            iframe.style.display = "block";

            // events
            window.addEventListener("message", (e) => {
                try {
                    const data = JSON.parse(e.data);
                    if (!data.vibrant_editor) return;
                    const useful_data = data.vibrant_editor;

                    // ...
                    if (useful_data.doc_title) {
                        document.title = useful_data.doc_title;
                    }

                    if (useful_data.click) {
                        document.getElementById(useful_data.click)?.click();
                    }
                } catch {}
            });

            iframe.addEventListener("load", () => {
                loading_modal.close();
                document.title = iframe.contentDocument?.title || doc_title;

                const href = iframe.contentWindow?.location.href;
                if (href !== editor_src) {
                    // handle editor close, the editor internally uses "about:blank" to close
                    iframe.remove();
                    document.title = doc_title;
                    file_editor_open = false;
                }
            });
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

// default export
export default {};
