//! Middleware for serving project assets
use actix_files::file_extension_to_mime;
use awc::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};

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
        // process response as normal
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            // get host
            let host = res.request().headers().get("host");

            // check host and return asset
            if host.is_some()
                && std::str::from_utf8(host.as_ref().unwrap().as_bytes())
                    .unwrap()
                    .contains(".get.")
            {
                // serve project asset
                let data = res
                    .request()
                    .app_data::<actix_web::web::Data<crate::db::AppData>>()
                    .unwrap();

                // ...
                let host = std::str::from_utf8(host.as_ref().unwrap().as_bytes()).unwrap();
                let host_split = host.split(".get.").collect::<Vec<&str>>();

                let project = host_split.get(0);
                if project.is_some() {
                    let project = project.unwrap();
                    let mut path = res.request().path().to_string();

                    // check path
                    if path == "/" {
                        path = String::from("/index.html");
                    } else if !path.starts_with("/") {
                        path = format!("/{}", path);
                    }

                    // fetch asset
                    let file = data
                        .db
                        .get_file_in_project(project.to_string(), path.clone())
                        .await;

                    if file.success == false {
                        let new_res = ServiceResponse::new(
                            res.request().clone(),
                            HttpResponse::NotFound().body("404: Not Found"),
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
