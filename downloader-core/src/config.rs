use clap::{arg, Command};

use super::manifest::{Location, Provider, DEFAULT_MANIFEST_URL};

#[derive(Debug)]
pub struct Config {
    pub manifest_location: Location,
    pub manifest_provider: Provider,
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
        let manifest = Location::parse(manifest_str)?;

        let provider = matches.get_one::<Provider>("provider").unwrap().clone();

        Ok(Config {
            manifest_location: manifest,
            manifest_provider: provider,
        })
    }
}
