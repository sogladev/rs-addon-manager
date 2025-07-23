use clap::{arg, Command};

use super::manifest::{Location, Provider};

// Define configuration constants based on feature flags
#[cfg(feature = "production")]
pub mod app_config {
    pub const DEFAULT_MANIFEST_URL: &str =
        "https://updater.project-epoch.net/api/v2/manifest?environment=production";
    pub const DEFAULT_FIGURE_TEXT: &str = "Project Epoch";
    pub const DEFAULT_DESCRIPTION: &str =
        "unofficial patch download utility - Sogladev\n\
        Bugs or issues: https://github.com/sogladev/rs-game-launcher\n\
        ----------------------------------------------------------------------------------------------------";
}

#[cfg(not(feature = "production"))]
pub mod app_config {
    pub const DEFAULT_MANIFEST_URL: &str = "http://localhost:8080/manifest.json";
    pub const DEFAULT_FIGURE_TEXT: &str = "Demo Launcher";
    pub const DEFAULT_DESCRIPTION: &str =
        "Demo version - For testing purposes only\n\
        Bugs or issues: https://github.com/sogladev/rs-game-launcher\n\
        ----------------------------------------------------------------------------------------------------";
}

pub use app_config::*;

#[derive(Debug)]
pub struct Config {
    pub manifest_location: Location,
    pub manifest_provider: Provider,
    pub figure_text: String,
    pub description: String,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let matches = Command::new("downloader-cli")
            .arg(arg!(-m --manifest <String> "Path to manifest.json file or URL (e.g., http://localhost:8080/manifest.json)")
                .default_value(DEFAULT_MANIFEST_URL))
            .arg(arg!(-p --provider <Provider> "Provider to use for downloads")
                .value_parser(clap::value_parser!(Provider))
                .default_value("cloudflare")
                .help("Available providers: cloudflare (Server #1), digitalocean (Server #2), none (Server #3 - Slowest)"))
            .get_matches();

        let manifest_str = matches.get_one::<String>("manifest").unwrap().to_string();
        let manifest_location = Location::parse(manifest_str)?;
        let manifest_provider = matches.get_one::<Provider>("provider").unwrap().clone();

        println!("{DEFAULT_FIGURE_TEXT}");
        println!("{DEFAULT_FIGURE_TEXT}");
        println!("{DEFAULT_FIGURE_TEXT}");

        Ok(Config {
            manifest_location,
            manifest_provider,
            figure_text: DEFAULT_FIGURE_TEXT.to_string(),
            description: DEFAULT_DESCRIPTION.to_string(),
        })
    }
}
