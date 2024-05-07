use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
/// `.vibrant.toml` file
pub struct Configuration {
    #[serde(default = "default_server")]
    pub server: String,
    #[serde(default = "default_auth_server")]
    pub auth_server: String,
    #[serde(default = "default_token")]
    pub token: String,
    // optionals
    pub name: Option<String>,
}

fn default_server() -> String {
    String::from("https://stellular.org")
}

fn default_auth_server() -> String {
    String::from("https://orion.stellular.net")
}

fn default_token() -> String {
    String::from("NO_TOKEN_PROVIDED")
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            server: String::from("https://stellular.org"),
            auth_server: String::from("https://orion.stellular.net"),
            token: String::from("NO_TOKEN_PROVIDED"),
            name: Option::None,
        }
    }
}

impl Configuration {
    pub fn get_config() -> Configuration {
        let contents = fs::read_to_string("./.vibrant.toml");

        if contents.is_ok() {
            toml::from_str::<Configuration>(&contents.unwrap()).unwrap()
        } else {
            Configuration::default()
        }
    }

    pub fn update_config(contents: Configuration) -> std::io::Result<()> {
        fs::write(
            "./.vibrant.toml",
            format!("# DO **NOT** SHARE THIS FILE! This is needed for the Vibrant server connection.\n{}", toml::to_string_pretty::<Configuration>(&contents).unwrap()),
        )
    }
}
