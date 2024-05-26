export function project_settings(
    metadata: { [key: string]: any },
    field: HTMLElement,
    _type: "project" | undefined
): void {
    if (_type === undefined) _type = "project";

    const update_form = document.getElementById(
        "update-form"
    ) as HTMLFormElement;

    const add_field = document.getElementById("add_field") as HTMLButtonElement;

    let current_property: string = "";
    let option_render: string = "";

    // handlers
    (window as any).change_current_property = (e: any) => {
        const selected = e.target.options[
            e.target.selectedIndex
        ] as HTMLOptionElement;

        if (selected) {
            current_property = selected.value;

            if (current_property === "file_privacy") {
                let meta_value = metadata[current_property];

                (globalThis as any).set_file_privacy = (e: any) => {
                    const selected = (
                        e.target.options[
                            e.target.selectedIndex
                        ] as HTMLOptionElement
                    ).value;

                    metadata[current_property] = selected;
                };

                // add button
                option_render = `<select class="round mobile:max" onchange="window.set_file_privacy(event);" style="width: 60%;">
                    <option value="Public" ${
                        meta_value === "Public" ? "selected" : ""
                    }>Public</option>

                    <option value="Confidential" ${
                        meta_value === "Confidential" ? "selected" : ""
                    }>Confidential</option>

                    <option value="Private" ${
                        meta_value === "Private" ? "selected" : ""
                    }>Private</option>
                </select>`;

                options = build_options(metadata, current_property);
                render_project_settings_fields(field, options, option_render); // rerender
                return;
            } else if (current_property === "clean_paths") {
                let meta_value = metadata[current_property];

                (globalThis as any).set_yes_no_option = (e: any) => {
                    const selected = (
                        e.target.options[
                            e.target.selectedIndex
                        ] as HTMLOptionElement
                    ).value;

                    metadata[current_property] = selected === "true";
                };

                // add button
                option_render = `<select class="round mobile:max" onchange="window.set_yes_no_option(event);" style="width: 60%;">
                    <option value="true" ${
                        meta_value === true ? "selected" : ""
                    }>Yes</option>

                    <option value="false" ${
                        meta_value === false ? "selected" : ""
                    }>No</option>
                </select>`;

                options = build_options(metadata, current_property);
                render_project_settings_fields(field, options, option_render); // rerender
                return;
            }

            // ...
            let meta_value = metadata[current_property];
            if (typeof meta_value === "string" || meta_value === null) {
                const use =
                    current_property === "script" ||
                    current_property === "page_template"
                        ? "textarea"
                        : "input";
                option_render = `<${use} 
                    type="text" 
                    name="${current_property}" 
                    placeholder="${current_property}" 
                    value="${use === "input" ? meta_value || "" : ""}" 
                    required 
                    oninput="window.project_settings_field_input(event);" 
                    class="round mobile:max"
                    style="width: 60%;"
                ${use === "textarea" ? `>${meta_value || ""}</textarea>` : "/>"}`;

                (window as any).project_settings_field_input = (e: any) => {
                    metadata[current_property] = e.target.value;
                };
            }
        }

        options = build_options(metadata, current_property);
        render_project_settings_fields(field, options, option_render); // rerender
    };

    // ...
    let options = build_options(metadata, current_property);
    render_project_settings_fields(field, options, option_render);

    // handle submit
    update_form.addEventListener("submit", async (e) => {
        e.preventDefault();

        const res = await fetch((globalThis as any).metadata_endpoint, {
            method: "POST",
            body: JSON.stringify({
                metadata,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });

        const json = await res.json();
        return alert(json.message);
    });

    // handle add field
    add_field.addEventListener("click", () => {
        const name = prompt("Enter field name:");
        if (!name) return;

        metadata[name] = "unknown";
        options = build_options(metadata, current_property);
        render_project_settings_fields(field, options, option_render);
    });
}

function build_options(
    metadata: { [key: string]: string },
    current_property: string
): string {
    let options: string = ""; // let mut options: String = "";

    for (let option of Object.entries(metadata)) {
        options += `<option value="${option[0]}" ${
            current_property === option[0] ? "selected" : ""
        }>${option[0]}</option>\n`;
    }

    return options;
}

function render_project_settings_fields(
    field: HTMLElement,
    options: string,
    option_render: string
): string {
    field.innerHTML = "";

    // render selector
    field.innerHTML += `<select class="round mobile:max" onchange="window.change_current_property(event);" style="width: 38%;">
        <option value="">Select a field to edit</option>
        ${options}
    </select>${option_render}`;

    // ...
    return "";
}

// default export
export default { project_settings };
