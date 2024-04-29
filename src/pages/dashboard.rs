use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::db::{PCreateProject, Project};

use super::base;
use askama::Template;

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct OffsetQueryProps {
    pub offset: Option<i32>,
}

#[derive(Template)]
#[template(path = "dashboard/auth_picker.html")]
struct AuthPickerTemplate {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    bundlrs: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "dashboard/homepage.html")]
struct DashboardTemplate {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    bundlrs: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "dashboard/new_project.html")]
struct NewProjectTemplate {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    bundlrs: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "dashboard/projects.html")]
struct ProjectsDashboardTemplate {
    projects: Vec<PCreateProject>,
    offset: i32,
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    bundlrs: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "dashboard/project/view.html")]
struct ProjectViewTemplate {
    project: Project,
    files: Vec<String>,
    asset_requests: String,
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    bundlrs: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "dashboard/project/editor.html")]
struct ProjectFileEditorTemplate {
    project: Project,
    file_path: String,
    file_content: String,
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    bundlrs: String,
    body_embed: String,
}

#[get("/dashboard")]
pub async fn dashboard_request(
    req: HttpRequest,
    data: web::Data<crate::db::AppData>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req, data).await;

    if token_user.is_none() {
        let base = base::get_base_values(token_user.is_some());
        return HttpResponse::NotAcceptable()
            .append_header(("Set-Cookie", set_cookie))
            .append_header(("Content-Type", "text/html"))
            .body(
                AuthPickerTemplate {
                    // required fields
                    auth_state: base.auth_state,
                    guppy: base.guppy,
                    bundlrs: base.bundlrs,
                    body_embed: base.body_embed,
                }
                .render()
                .unwrap(),
            );
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            DashboardTemplate {
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                bundlrs: base.bundlrs,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

#[get("/dashboard/project/new")]
pub async fn new_project_request(
    req: HttpRequest,
    data: web::Data<crate::db::AppData>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req, data).await;

    if token_user.is_none() {
        let base = base::get_base_values(token_user.is_some());
        return HttpResponse::NotAcceptable()
            .append_header(("Set-Cookie", set_cookie))
            .append_header(("Content-Type", "text/html"))
            .body(
                AuthPickerTemplate {
                    // required fields
                    auth_state: base.auth_state,
                    guppy: base.guppy,
                    bundlrs: base.bundlrs,
                    body_embed: base.body_embed,
                }
                .render()
                .unwrap(),
            );
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            NewProjectTemplate {
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                bundlrs: base.bundlrs,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

#[get("/dashboard/projects")]
pub async fn projects_dashboard_request(
    req: HttpRequest,
    data: web::Data<crate::db::AppData>,
    info: web::Query<OffsetQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req, data.clone()).await;

    if token_user.is_none() {
        let base = base::get_base_values(token_user.is_some());
        return HttpResponse::NotAcceptable()
            .append_header(("Set-Cookie", set_cookie))
            .append_header(("Content-Type", "text/html"))
            .body(
                AuthPickerTemplate {
                    // required fields
                    auth_state: base.auth_state,
                    guppy: base.guppy,
                    bundlrs: base.bundlrs,
                    body_embed: base.body_embed,
                }
                .render()
                .unwrap(),
            );
    }

    // fetch projects
    let projects = data
        .db
        .get_projects_by_owner_limited(
            token_user.clone().unwrap().payload.unwrap().user.username,
            info.offset,
        )
        .await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            ProjectsDashboardTemplate {
                projects: projects.payload.unwrap(),
                offset: if info.offset.is_some() {
                    info.offset.unwrap()
                } else {
                    0
                },
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                bundlrs: base.bundlrs,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

#[get("/dashboard/project/{project:.*}")]
pub async fn project_view_request(
    req: HttpRequest,
    data: web::Data<crate::db::AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("project").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        let base = base::get_base_values(token_user.is_some());
        return HttpResponse::NotAcceptable()
            .append_header(("Set-Cookie", set_cookie))
            .append_header(("Content-Type", "text/html"))
            .body(
                AuthPickerTemplate {
                    // required fields
                    auth_state: base.auth_state,
                    guppy: base.guppy,
                    bundlrs: base.bundlrs,
                    body_embed: base.body_embed,
                }
                .render()
                .unwrap(),
            );
    }

    // fetch project
    let project = data.db.get_project_by_id(project_name.to_string()).await;

    if !project.success {
        return super::errors::error404(req, data).await;
    }

    // fetch project files
    let files = data.db.get_project_files(project_name.to_string()).await;

    if !files.success {
        return super::errors::error404(req, data).await;
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            ProjectViewTemplate {
                project: project.payload.unwrap(),
                files: files.payload,
                asset_requests: data
                    .db
                    .base
                    .cachedb
                    .get(format!("billing:requests:{}", project_name))
                    .await
                    .unwrap_or("0".to_string()),
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                bundlrs: base.bundlrs,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

#[get("/dashboard/project/{project:.*}/edit/{path:.*}")]
pub async fn project_file_editor_request(
    req: HttpRequest,
    data: web::Data<crate::db::AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("project").unwrap();
    let path = req.match_info().get("path").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        let base = base::get_base_values(token_user.is_some());
        return HttpResponse::NotAcceptable()
            .append_header(("Set-Cookie", set_cookie))
            .append_header(("Content-Type", "text/html"))
            .body(
                AuthPickerTemplate {
                    // required fields
                    auth_state: base.auth_state,
                    guppy: base.guppy,
                    bundlrs: base.bundlrs,
                    body_embed: base.body_embed,
                }
                .render()
                .unwrap(),
            );
    }

    // fetch project
    let project = data.db.get_project_by_id(project_name.to_string()).await;

    if !project.success {
        return super::errors::error404(req, data).await;
    }

    // fetch project files
    let file = data
        .db
        .get_file_in_project(project_name.to_string(), path.to_string())
        .await;

    if !file.success {
        return super::errors::error404(req, data).await;
    }

    let payload = file.payload.unwrap();
    let as_str = std::str::from_utf8(&payload)
        .unwrap_or("Failed to read file as UTF-8 string");

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            ProjectFileEditorTemplate {
                project: project.payload.unwrap(),
                file_path: path.to_string(),
                file_content: as_str.to_string(),
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                bundlrs: base.bundlrs,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
