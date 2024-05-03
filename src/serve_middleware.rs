//! Middleware for serving project assets
use actix_files::file_extension_to_mime;
use awc::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};

use crate::pages::base;

pub struct ServeAssets;

impl<S, B> Transform<S, ServiceRequest> for ServeAssets
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ServeMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ServeMiddleware { service }))
    }
}

pub struct ServeMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ServeMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let site_host = std::env::var("HOST");
        let site_host_no_tld = std::env::var("HOST_NO_TLD");

        // process response as normal
        let cookie = req.request().cookie("__Secure-Token");
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            // get host
            let host = res.request().headers().get("host");

            // check host and return asset
            if host.is_some() && site_host.is_ok() && site_host_no_tld.is_ok()
            // && std::str::from_utf8(host.as_ref().unwrap().as_bytes())
            //     .unwrap()
            //     .contains(".get.")
            {
                let site_host = site_host.unwrap();
                let site_host_no_tld = site_host_no_tld.unwrap();

                // serve project asset
                let data = res
                    .request()
                    .app_data::<actix_web::web::Data<crate::db::AppData>>()
                    .unwrap();

                // ...
                let host = std::str::from_utf8(host.as_ref().unwrap().as_bytes()).unwrap();
                // let host_split = host.split(".get.").collect::<Vec<&str>>();
                let host_split = host.split(".").collect::<Vec<&str>>(); // we're splitting by a single period here, so projects CANNOT contain "." in their names ...
                                                                         // this also means that we can host internal pages under a "nested subdomain" (one.two.three.host)

                let project = host_split.get(0);
                if project.is_some() {
                    let project = project
                        .unwrap()
                        .replace("https://", "")
                        .replace("http://", "");

                    let project = project.as_str();

                    // make sure project is not the host and is not "www"
                    if [host, &site_host, &site_host_no_tld, "www", ""].contains(&project) {
                        return Ok(res.map_into_left_body());
                    }

                    // ...
                    let mut path = res.request().path().to_string();

                    // check path
                    if path == "/" {
                        path = String::from("/index.html");
                    } else if !path.starts_with("/") {
                        path = format!("/{}", path);
                    }

                    // verify auth status
                    let (set_cookie, _, token_user) =
                        base::check_auth_status_with_cookie(cookie, data.clone()).await;

                    // fetch asset
                    let file = data
                        .db
                        .get_file_in_project(
                            project.to_string(),
                            path.clone(),
                            if token_user.is_some() {
                                let user = token_user.unwrap().payload.unwrap();
                                Option::Some(user.user.username)
                            } else {
                                Option::None
                            },
                            false,
                        )
                        .await;

                    if file.success == false {
                        let new_res = ServiceResponse::new(
                            res.request().clone(),
                            HttpResponse::NotAcceptable()
                                .append_header(("Content-Type", "text/html"))
                                .body(format!("<!DOCTYPE html>

                                <html lang=\"en\">
                                    <head>
                                        <meta charset=\"UTF-8\" />
                                        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
                                        <title>Error! (Vibrant)</title>

                                        <link rel=\"stylesheet\" href=\"//{site_host}/static/style.css\" />
                                    </head>
                                
                                    <body>
                                        <main class=\"small flex flex-column g-4\">
                                            <div class=\"card secondary border round full flex justify-center align-center\">
                                                <h3 class=\"no-margin text-center\">{}</h3>
                                            </div>

                                            <div class=\"flex justify-center footernav\">
                                                <span class=\"item\"><a href=\"/\">Root</a></span>
                                                <span class=\"item\"><a href=\"//{site_host}\">🌸 Homepage</a></span>
                                                <span class=\"item\"><a href=\"https://code.stellular.org/stellular/vibrant\">Source Code</a></span>
                                            </div>
                                        </main>
                                    </body>
                                </html>", file.message)),
                        )
                        .map_into_right_body();

                        return Ok(new_res);
                    }

                    // get file extension from path
                    let ext = path
                        .split(".")
                        .collect::<Vec<&str>>()
                        .pop()
                        .unwrap_or("txt");

                    // return
                    let new_res = ServiceResponse::new(
                        res.request().clone(),
                        HttpResponse::Ok()
                            .append_header(("Set-Cookie", set_cookie))
                            .append_header(("Content-Type", file_extension_to_mime(ext)))
                            .body(file.payload.unwrap()),
                    )
                    .map_into_right_body();

                    return Ok(new_res);
                }
            }

            // normal res
            Ok(res.map_into_left_body())
        })
    }
}
