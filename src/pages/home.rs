use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use super::base;
use askama::Template;

#[derive(Template)]
#[template(path = "homepage.html")]
struct HomeTemplate {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    body_embed: String,
}

#[get("/")]
pub async fn home_request(req: HttpRequest, data: web::Data<crate::db::AppData>) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req, data).await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            HomeTemplate {
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
