use actix_files as fs;
use actix_web::{web, App, HttpServer};
use db::Database;
use dotenv;

pub mod api;
pub mod db;
pub mod pages;

use crate::db::AppData;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // configuration
    let args: Vec<String> = dorsal::collect_arguments();

    let port_search: Option<String> = dorsal::get_named_argument(&args, "port");
    let mut port: u16 = 8080;

    if port_search.is_some() {
        port = port_search.unwrap().parse::<u16>().unwrap();
    }

    let static_dir_flag: Option<String> = dorsal::get_named_argument(&args, "static-dir");

    // create database
    let db_type: Option<String> = dorsal::get_named_argument(&args, "db-type");
    let db_host: Option<String> = dorsal::get_var("DB_HOST");
    let db_user: Option<String> = dorsal::get_var("DB_USER");
    let db_pass: Option<String> = dorsal::get_var("DB_PASS");
    let db_name: Option<String> = dorsal::get_var("DB_NAME");

    let db_is_other: bool = db_type
        .clone()
        .is_some_and(|x| (x == String::from("postgres")) | (x == String::from("mysql")));

    if db_is_other && (db_user.is_none() | db_pass.is_none() | db_name.is_none()) {
        panic!("Missing required database config settings!");
    }

    let db: Database = Database::new(dorsal::DatabaseOpts {
        _type: db_type,
        host: db_host,
        user: if db_is_other {
            db_user.unwrap()
        } else {
            String::new()
        },
        pass: if db_is_other {
            db_pass.unwrap()
        } else {
            String::new()
        },
        name: if db_is_other {
            db_name.unwrap()
        } else {
            String::new()
        },
    })
    .await;

    db.init().await;

    // start server
    println!("Starting server at: http://localhost:{port}");
    HttpServer::new(move || {
        let client = awc::Client::default();
        let data = web::Data::new(AppData {
            db: db.clone(),
            http_client: client,
        });

        let cors = actix_cors::Cors::default().send_wildcard();

        App::new()
            .app_data(web::Data::clone(&data))
            // middleware
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors)
            // static dir
            .service(
                fs::Files::new(
                    "/static",
                    if static_dir_flag.is_some() {
                        static_dir_flag.as_ref().unwrap()
                    } else {
                        "./static"
                    },
                )
                .show_files_listing(),
            )
            // docs
            .service(fs::Files::new("/api/docs", "./target/doc").show_files_listing())
            // POST api
            .service(crate::api::auth::callback_request)
            .service(crate::api::projects::create_request)
            .service(crate::api::projects::edit_script_request)
            // GET api
            .service(crate::api::auth::logout)
            .service(crate::api::projects::get_script_request)
            // GET root
            .service(crate::pages::home::home_request)
            .service(crate::pages::dashboard::dashboard_request)
            .service(crate::pages::dashboard::new_project_request)
            .service(crate::pages::dashboard::projects_dashboard_request)
            // GET project
            .service(crate::pages::dashboard::project_view_request)
            // ERRORS
            .default_service(web::to(|req, data| async {
                return crate::pages::errors::error404(req, data).await;
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
