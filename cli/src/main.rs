use awc::{cookie::Cookie, http::StatusCode, Client};
use base64::Engine;
use clap::{ArgAction, Parser, Subcommand};

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultReturn<B> {
    pub success: bool,
    pub message: String,
    pub payload: B,
}

// cli
#[derive(Parser, Debug)]
#[command(version, about, long_about = Option::Some("Vibrant Sync CLI for managing Vibrant projects remotely"))]
#[command(propagate_version = true)]
struct Vibsync {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage authentication (SECONDARY TOKEN ONLY)
    Login { token: String },
    /// Link project
    Init { project: String },
    /// Pull project
    Pull {},
    /// Push files
    Push {
        #[arg(short, long, action = ArgAction::SetTrue)]
        create: bool,
        files: Vec<String>,
    },
    /// Remove files
    Remove { files: Vec<String> },
}

// ...
pub mod config;

#[actix_rt::main]
async fn main() {
    // init
    let args = Vibsync::parse();
    let client = Client::default();

    // get current config
    let cnf = config::Configuration::get_config();

    // match commands
    match &args.command {
        // login
        Commands::Login { token } => {
            // check token validity by attempting to login
            let res = client
                .post(format!("{}/api/auth/login-st", cnf.auth_server))
                .timeout(std::time::Duration::from_millis(10_000))
                .append_header(("Content-Type", "application/json"))
                // .cookie(Cookie::new())
                .send_body(
                    serde_json::to_string(&json!({
                        "uid": token
                    }))
                    .unwrap(),
                )
                .await;

            if res.is_err() | (res.as_ref().unwrap().status() == StatusCode::NOT_ACCEPTABLE) {
                no("Failed to send request! Token may be invalid or the server may be unreachable.");
            }

            // update configuration
            let res = config::Configuration::update_config(config::Configuration {
                server: cnf.server,
                auth_server: cnf.auth_server,
                token: token.to_string(),
                ..Default::default()
            });

            if res.is_err() {
                no("Failed to write token!");
            } else {
                yes("Token written to configuration!");
            }
        }
        // init
        Commands::Init { project } => {
            // make sure we haven't already set a project
            if cnf.name.is_some() {
                no("A project has already been set!");
            }

            // add to config
            // update configuration
            let res = config::Configuration::update_config(config::Configuration {
                server: cnf.server,
                auth_server: cnf.auth_server,
                token: cnf.token,
                name: Option::Some(project.to_string()),
                ..Default::default()
            });

            if res.is_err() {
                no("Failed to write token!");
            } else {
                yes("Project configuration added!");
            }
        }
        // pull
        Commands::Pull {} => {
            // make sure project is set
            if cnf.name.is_none() {
                no("Please set a project first!");
            }

            if cnf.token == "NO_TOKEN_PROVIDED" {
                no("Please set a token first!");
            }

            let project = cnf.name.unwrap();
            let token_cookie = Cookie::new("__Secure-Token", cnf.token);

            // get file list
            maybe("Requesting file list...");
            let res = client
                .get(format!("{}/api/v1/project/{}/files", cnf.server, project))
                .timeout(std::time::Duration::from_millis(10_000))
                .append_header(("Content-Type", "application/json"))
                .cookie(token_cookie.clone())
                .send()
                .await;

            if res.is_err() | (res.as_ref().unwrap().status() == StatusCode::NOT_ACCEPTABLE) {
                no("Failed to send request! An error may have occurred or the server may be unreachable.");
            }

            let mut res = res.unwrap();

            // fill body content
            let body_ = res.body().limit(1_000_000).await;

            if body_.is_err() {
                no("Failed to read response body!");
            }

            let binding = body_.unwrap();
            let body_ = std::str::from_utf8(&binding).unwrap();
            let files = serde_json::from_str::<DefaultReturn<Vec<String>>>(body_).unwrap();

            // download files
            maybe(&format!("Project contains {} files!", files.payload.len()));
            maybe("Downloading files...");

            for file in files.payload {
                almost(&format!("Pulling {}", file));
                let res = client
                    .get(format!(
                        "{}/api/v1/project/{}/files/{}",
                        cnf.server, project, file
                    ))
                    .timeout(std::time::Duration::from_millis(10_000))
                    .append_header(("Content-Type", "application/json"))
                    .cookie(token_cookie.clone())
                    .send()
                    .await;

                if res.is_err() | (res.as_ref().unwrap().status() == StatusCode::NOT_ACCEPTABLE) {
                    no("Failed to send request! An error may have occurred or the server may be unreachable.");
                }

                let mut res = res.unwrap();

                // fill body content
                let body_ = res.body().limit(1_000_000).await;

                if body_.is_err() {
                    no("Failed to read response body!");
                }

                let binding = body_.unwrap();
                let body_ = std::str::from_utf8(&binding).unwrap();

                // write file
                // files start with "/" by default, so we'll just make that relative!
                let path = format!(".{file}");
                let path = std::path::Path::new(&path);

                let res = std::fs::create_dir_all(path.parent().unwrap());

                if res.is_err() {
                    no(&format!("ERR: {}", res.err().unwrap()));
                }

                std::fs::write(path, body_).unwrap_or_else(|_| {
                    no("Error while writing file!");
                });
            }

            yes("Successfully pulled project!");
        }
        // push
        Commands::Push { create, files } => {
            // make sure project is set
            if cnf.name.is_none() {
                no("Please set a project first!");
            }

            if cnf.token == "NO_TOKEN_PROVIDED" {
                no("Please set a token first!");
            }

            if files.len() == 0 {
                no("Please specify some files first!");
            }

            let project = cnf.name.unwrap();
            let token_cookie = Cookie::new("__Secure-Token", cnf.token);

            // handle create
            if *create {
                maybe("Selected \"create\" mode...");

                // files should be specified relatively, so without the leading slash
                for file in files {
                    // attempt to read file
                    let content = std::fs::read_to_string(file).unwrap_or_else(|_| {
                        no("Error fetching local file content to upload!");
                        String::new()
                    });

                    // attempt to turn to base64
                    let base64 = base64::engine::general_purpose::STANDARD.encode(content);

                    // push to server
                    almost(&format!("Pushing {}", file));

                    let res = client
                        .post(format!(
                            "{}/api/v1/project/{}/files/{}",
                            cnf.server, project, file
                        ))
                        .timeout(std::time::Duration::from_millis(10_000))
                        .append_header(("Content-Type", "application/json"))
                        .cookie(token_cookie.clone())
                        .send_body(
                            serde_json::to_string(&json!({
                                "content": base64
                            }))
                            .unwrap(),
                        )
                        .await;

                    if res.is_err() | (res.as_ref().unwrap().status() == StatusCode::NOT_ACCEPTABLE)
                    {
                        no("Failed to send request! An error may have occurred or the server may be unreachable.");
                    }
                }

                yes(&format!("Pushed {} files!", files.len()));
            }
            // handle update
            else {
                maybe("Selected \"update\" mode...");

                // files should be specified relatively, so without the leading slash
                for file in files {
                    // attempt to read file
                    let content = std::fs::read_to_string(file).unwrap();

                    // attempt to turn to base64
                    let base64 = base64::engine::general_purpose::STANDARD.encode(content);

                    // push to server
                    almost(&format!("Pushing {}", file));

                    let res = client
                        .put(format!(
                            "{}/api/v1/project/{}/files/{}",
                            cnf.server, project, file
                        ))
                        .timeout(std::time::Duration::from_millis(10_000))
                        .append_header(("Content-Type", "application/json"))
                        .cookie(token_cookie.clone())
                        .send_body(
                            serde_json::to_string(&json!({
                                "content": base64
                            }))
                            .unwrap(),
                        )
                        .await;

                    if res.is_err() | (res.as_ref().unwrap().status() == StatusCode::NOT_ACCEPTABLE)
                    {
                        no("Failed to send request! An error may have occurred or the server may be unreachable.");
                    }
                }

                yes(&format!("Updated {} files!", files.len()));
            }
        }
        // remove
        Commands::Remove { files } => {
            // make sure project is set
            if cnf.name.is_none() {
                no("Please set a project first!");
            }

            if cnf.token == "NO_TOKEN_PROVIDED" {
                no("Please set a token first!");
            }

            if files.len() == 0 {
                no("Please specify some files first!");
            }

            let project = cnf.name.unwrap();
            let token_cookie = Cookie::new("__Secure-Token", cnf.token);

            // files should be specified relatively, so without the leading slash
            for file in files {
                // push to server
                almost(&format!("Removing {}", file));

                let res = client
                    .delete(format!(
                        "{}/api/v1/project/{}/files//{}",
                        cnf.server, project, file
                    ))
                    .timeout(std::time::Duration::from_millis(10_000))
                    .append_header(("Content-Type", "application/json"))
                    .cookie(token_cookie.clone())
                    .send()
                    .await;

                if res.is_err() | (res.as_ref().unwrap().status() == StatusCode::NOT_ACCEPTABLE) {
                    no("Failed to send request! An error may have occurred or the server may be unreachable.");
                }
            }

            yes(&format!("Removed {} files!", files.len()));
        }
    }
}

fn no(msg: &str) -> () {
    println!("\x1b[91m{}\x1b[0m", format!("✘ ⎹ {msg}"));
    std::process::exit(1);
}

fn yes(msg: &str) -> () {
    println!("\x1b[92m{}\x1b[0m", format!("✔ ⎹ {msg}"));
    std::process::exit(0);
}

fn maybe(msg: &str) -> () {
    println!("🛈 ⎹ {}", msg);
}

fn almost(msg: &str) -> () {
    println!("\x1b[94m{}\x1b[0m", format!("🛈 ⎹ {msg}"));
}
