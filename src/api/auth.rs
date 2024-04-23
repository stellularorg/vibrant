use crate::db::AppData;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct CallbackQueryProps {
    pub uid: Option<String>, // this uid will need to be sent to the client as a token
                             // the uid will also be sent to the client as a token on GUPPY_ROOT, meaning we'll have signed in here and there!
}

#[get("/api/auth/callback")]
pub async fn callback_request(info: web::Query<CallbackQueryProps>) -> impl Responder {
    let set_cookie = if info.uid.is_some() {
        format!("__Secure-Token={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}", info.uid.as_ref().unwrap(), 60 * 60 * 24 * 365)
    } else {
        String::new()
    };

    // return
    return HttpResponse::Ok()
        .append_header((
            "Set-Cookie",
            if info.uid.is_some() { &set_cookie } else { "" },
        ))
        .append_header(("Content-Type", "text/html"))
        .body(
            "<head>
                <meta http-equiv=\"Refresh\" content=\"0; URL=/\" />
            </head>",
        );
}

#[get("/api/auth/logout")]
pub async fn logout(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let cookie = req.cookie("__Secure-Token");

    if cookie.is_none() {
        return HttpResponse::NotAcceptable().body("Missing token");
    }

    let res = data
        .db
        .auth
        .get_user_by_unhashed(cookie.unwrap().value().to_string()) // if the user is returned, that means the ID is valid
        .await;

    if !res.success {
        return HttpResponse::NotAcceptable().body("Invalid token");
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0"))
        .append_header(("Content-Type", "text/plain"))
        .body("You have been signed out. You can now close this tab.");
}
