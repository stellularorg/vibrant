use crate::db::{AppData, PCreateProject};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

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
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .append_header(("Set-Cookie", set_cookie))
        .body(serde_json::to_string(&res).unwrap());
}
