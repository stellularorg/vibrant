use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use super::base;
use askama::Template;

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct SQLQueryProps {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub mode: String,
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
#[template(path = "admin/homepage.html")]
struct DashboardTemplate {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    bundlrs: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "admin/sql.html")]
struct SQLViewerTemplate {
    res: Vec<Vec<(String, String)>>,
    query: String,
    mode: String,
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    bundlrs: String,
    body_embed: String,
}

#[get("/dashboard/admin")]
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

    let user = token_user.as_ref().unwrap().payload.as_ref().unwrap();

    if !user.level.permissions.contains(&"VIB:Admin".to_string()) {
        return HttpResponse::NotAcceptable()
            .append_header(("Content-Type", "text/plain"))
            .body("You are not allowed to view this page.");
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

#[get("/dashboard/admin/sql")]
pub async fn sql_viewer_request(
    req: HttpRequest,
    data: web::Data<crate::db::AppData>,
    info: web::Query<SQLQueryProps>,
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

    let user = token_user.as_ref().unwrap().payload.as_ref().unwrap();

    if !user.level.permissions.contains(&"VIB:Admin:SQL".to_string()) {
        return HttpResponse::NotAcceptable()
            .append_header(("Content-Type", "text/plain"))
            .body("You are not allowed to view this page.");
    }

    // ...
    let res = if info.mode == "" {
        let mut out = Vec::new();

        for x in data.db.general_query(info.query.clone()).await.payload {
            // sort columns
            out.push(data.db.sort_hashmap_by_keys::<String>(x))
        }

        out
    } else {
        Vec::new()
    };

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            SQLViewerTemplate {
                res,
                query: info.query.clone(),
                mode: info.mode.clone(),
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
