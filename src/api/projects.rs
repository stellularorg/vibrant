use crate::db::{AppData, PCreateProject};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::pages::base;

#[post("/api/v1/projects")]
/// Create a new project ([`crate::db::Database::create_project`])
pub async fn create_request(
    req: HttpRequest,
    mut body: web::Json<PCreateProject>,
    data: web::Data<AppData>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req, data.clone()).await;

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body("An account is required to create projects.");
    }

    // create paste
    let res = data
        .db
        .create_project(
            &mut body.0,
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().user.username)
            } else {
                Option::None
            },
            data.daemon.clone(),
            data.port.clone(),
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .append_header(("Set-Cookie", set_cookie))
        .body(serde_json::to_string(&res).unwrap());
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PEditScript {
    pub script: String,
}

#[post("/api/v1/project/{name:.*}/script")]
/// Edit a project's build script
pub async fn edit_script_request(
    req: HttpRequest,
    body: web::Json<PEditScript>,
    data: web::Data<AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body("An account is required to edit projects.");
    }

    // get project
    let project = data.db.get_project_by_id(project_name.to_string()).await;

    if project.success == false {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(serde_json::to_string(&project).unwrap());
    }

    let project = project.payload.unwrap();
    let mut metadata = project.metadata;
    metadata.script = body.script.clone();

    // update metadata
    let res = data
        .db
        .edit_project_metadata_by_name(
            project_name.to_string(),
            metadata,
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().user.username)
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .append_header(("Set-Cookie", set_cookie))
        .body(serde_json::to_string(&res).unwrap());
}

#[get("/api/v1/project/{name:.*}/script")]
/// Get a project's build script
pub async fn get_script_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();

    // get project
    let project = data.db.get_project_by_id(project_name.to_string()).await;

    if project.success == false {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(serde_json::to_string(&project).unwrap());
    }

    let project = project.payload.unwrap();
    let metadata = project.metadata;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/x-sh"))
        .body(metadata.script);
}

#[delete("/api/v1/project/{name:.*}")]
/// Delete a project given its `name`
pub async fn delete_project_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body("An account is required to edit projects.");
    }

    // delete project
    let res = data
        .db
        .delete_project(
            project_name.to_string(),
            Option::Some(token_user.unwrap().payload.unwrap().user.username),
            data.daemon.clone(),
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .append_header(("Set-Cookie", set_cookie))
        .body(serde_json::to_string::<dorsal::DefaultReturn<Option<String>>>(&res).unwrap());
}
