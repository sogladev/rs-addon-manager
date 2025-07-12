use clap::{arg, Command};
use std::str::FromStr;

use super::manifest::{Location, Provider};

#[derive(Debug)]
pub struct Config {
    pub manifest_location: Location,
    pub manifest_provider: Provider,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let matches = Command::new("rs_manifest_patcher")
            .arg(arg!(-m --manifest <String> "Path to manifest.json file or URL (e.g., http://localhost:8080/manifest.json)")
                .default_value("https://updater.project-epoch.net/api/v2/manifest?environment=production"))
            .arg(arg!(-p --provider <String> "Provider to use for downloads")
                .value_parser(Provider::known_keys())
                .default_value("cloudflare")
                .help("Available providers: cloudflare (Server #1), digitalocean (Server #2), none (Server #3 - Slowest)"))
            .get_matches();

        let manifest_str = matches.get_one::<String>("manifest").unwrap().to_string();
        let manifest = Location::parse(manifest_str)?;

        let provider_str = matches.get_one::<String>("provider").unwrap().as_str();
        let provider = Provider::from_str(provider_str).unwrap();

        Ok(Config {
            manifest_location: manifest,
            manifest_provider: provider,
        })
    }
}
