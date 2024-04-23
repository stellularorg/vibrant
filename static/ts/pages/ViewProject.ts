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

// default export
export default {};
