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
    body_embed: String,
}

#[derive(Template)]
#[template(path = "dashboard/homepage.html")]
struct DashboardTemplate {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "dashboard/new_project.html")]
struct NewProjectTemplate {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
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
                    .unwrap(),
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
