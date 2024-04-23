use crate::db::AppData;

use super::base;
use actix_web::{web, HttpRequest, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "general/404.html")]
struct Error404Template {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    body_embed: String,
}

pub async fn error404(req: HttpRequest, data: web::Data<AppData>) -> HttpResponse {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::NotFound()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            Error404Template {
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
