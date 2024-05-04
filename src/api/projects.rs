use crate::db::{AppData, PCreateProject};
use actix_files::file_extension_to_mime;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use dorsal::DefaultReturn;
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

#[derive(Clone, Serialize, Deserialize)]
pub struct PAddFile {
    /// base64 file content
    pub content: String,
}

#[get("/api/v1/project/{name:.*}/files")]
/// Project file listing
pub async fn get_project_files_request(
    req: HttpRequest,
    data: web::Data<AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // if token_user.is_none() {
    //     return HttpResponse::NotAcceptable().body("An account is required to list project files.");
    // }

    // ...
    let res = data
        .db
        .get_project_files(
            project_name.to_string(),
            if token_user.is_some() {
                let user = token_user.unwrap().payload.unwrap();
                Option::Some(user.user.username)
            } else {
                Option::None
            },
            false,
        )
        .await;

    if res.success == false {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/plain"))
            .append_header(("Set-Cookie", set_cookie))
            .body(res.message);
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .append_header(("Set-Cookie", set_cookie))
        .body(serde_json::to_string::<DefaultReturn<Vec<String>>>(&res).unwrap());
}

#[get("/api/v1/project/{name:.*}/files/{path:.*}")]
/// Read a file from a project
pub async fn read_file_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();
    let path = req.match_info().get("path").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let res = data
        .db
        .get_file_in_project(
            project_name.to_string(),
            path.to_string(),
            if token_user.is_some() {
                let user = token_user.unwrap().payload.unwrap();
                Option::Some(user.user.username)
            } else {
                Option::None
            },
            false,
        )
        .await;

    if res.success == false {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/plain"))
            .append_header(("Set-Cookie", set_cookie))
            .body(res.message);
    }

    // incr project requests
    data.db
        .incr_project_requests(project_name.to_string())
        .await;

    // get file extension from path
    let ext = path
        .split(".")
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or("txt");

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", file_extension_to_mime(ext)))
        .append_header(("Set-Cookie", set_cookie))
        .body(res.payload.unwrap());
}

#[get("/{name:.*}")]
/// Read a file from a project
pub async fn read_project_global_request(
    req: HttpRequest,
    data: web::Data<AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();
    let path = "/index.html";

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let res = data
        .db
        .get_file_in_project(
            project_name.to_string(),
            path.to_string(),
            if token_user.is_some() {
                let user = token_user.unwrap().payload.unwrap();
                Option::Some(user.user.username)
            } else {
                Option::None
            },
            false,
        )
        .await;

    if res.success == false {
        return crate::pages::errors::error404(req, data).await;
    }

    // incr project requests
    data.db
        .incr_project_requests(project_name.to_string())
        .await;

    // get file extension from path
    let ext = path
        .split(".")
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or("txt");

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", file_extension_to_mime(ext)))
        .append_header(("Set-Cookie", set_cookie))
        .body(res.payload.unwrap());
}

#[get("/{name:.*}/{path:.*}")]
/// Read a file from a project
pub async fn read_file_global_request(
    req: HttpRequest,
    data: web::Data<AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();
    let path = req.match_info().get("path").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let res = data
        .db
        .get_file_in_project(
            project_name.to_string(),
            path.to_string(),
            if token_user.is_some() {
                let user = token_user.unwrap().payload.unwrap();
                Option::Some(user.user.username)
            } else {
                Option::None
            },
            false,
        )
        .await;

    if res.success == false {
        return crate::pages::errors::error404(req, data).await;
    }

    // incr project requests
    data.db
        .incr_project_requests(project_name.to_string())
        .await;

    // get file extension from path
    let ext = path
        .split(".")
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or("txt");

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", file_extension_to_mime(ext)))
        .append_header(("Set-Cookie", set_cookie))
        .body(res.payload.unwrap());
}

#[post("/api/v1/project/{name:.*}/files/{path:.*}")]
/// Insert a file into a project
pub async fn insert_file_request(
    req: HttpRequest,
    body: web::Json<PAddFile>,
    data: web::Data<AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();
    let path = req.match_info().get("path").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body("An account is required to edit projects.");
    }

    // check size
    // file must be less than or equal to 1 MB
    let content_length = req.headers().get("Content-Length");

    if content_length.is_none()
        | (std::str::from_utf8(content_length.unwrap().as_bytes())
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 1_048_576)
    {
        return HttpResponse::PayloadTooLarge()
            .append_header(("Content-Type", "text/plain"))
            .append_header(("Set-Cookie", set_cookie))
            .body("Payload is too large.");
    }

    // ...
    let res = data
        .db
        .store_file_in_project(
            project_name.to_string(),
            path.to_string(),
            body.content.clone(),
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

#[put("/api/v1/project/{name:.*}/files/{path:.*}")]
/// Update a file in a project
pub async fn update_file_request(
    req: HttpRequest,
    body: web::Json<PAddFile>,
    data: web::Data<AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();
    let path = req.match_info().get("path").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body("An account is required to edit projects.");
    }

    // check size
    // file must be less than or equal to 1 MB
    let content_length = req.headers().get("Content-Length");

    if content_length.is_none()
        | (std::str::from_utf8(content_length.unwrap().as_bytes())
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 1_048_576)
    {
        return HttpResponse::PayloadTooLarge()
            .append_header(("Content-Type", "text/plain"))
            .append_header(("Set-Cookie", set_cookie))
            .body("Payload is too large.");
    }

    // ...
    let res = data
        .db
        .update_file_in_project(
            project_name.to_string(),
            path.to_string(),
            body.content.clone(),
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

#[delete("/api/v1/project/{name:.*}/files/{path:.*}")]
/// Delete a file from a project
pub async fn delete_file_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();
    let path = req.match_info().get("path").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body("An account is required to edit projects.");
    }

    // ...
    let res = data
        .db
        .delete_file_in_project(
            project_name.to_string(),
            path.to_string(),
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

#[derive(Clone, Serialize, Deserialize)]
pub struct PMoveFile {
    pub path: String,
}

#[post("/api/v1/project/{name:.*}/files:mv/{path:.*}")]
/// Move a file in a project
pub async fn move_file_request(
    req: HttpRequest,
    body: web::Json<PMoveFile>,
    data: web::Data<AppData>,
) -> impl Responder {
    let project_name = req.match_info().get("name").unwrap();
    let path = req.match_info().get("path").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body("An account is required to edit projects.");
    }

    // ...
    let res = data
        .db
        .move_file_in_project(
            project_name.to_string(),
            path.to_string(),
            body.path.clone(),
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
