use clap::{arg, Command};
use downloader_core::{
    constants::{CURRENT_VERSION, DEFAULT_DESCRIPTION, DEFAULT_FIGURE_TEXT, DEFAULT_MANIFEST_URL},
    manifest::{Location, Provider},
};

#[derive(Debug)]
pub struct Config {
    pub manifest_location: Location,
    pub manifest_provider: Provider,
    pub figure_text: String,
    pub description: String,
    pub verbose: bool,
    pub yes: bool,
    pub check_updates: bool,
}

impl Config {
    pub fn new(
        manifest_location: Location,
        manifest_provider: Provider,
        verbose: bool,
        yes: bool,
        check_updates: bool,
    ) -> Self {
        Config {
            manifest_location,
            manifest_provider,
            figure_text: DEFAULT_FIGURE_TEXT.to_string(),
            description: DEFAULT_DESCRIPTION.to_string(),
            verbose,
            yes,
            check_updates,
        }
    }

    pub fn build_config() -> Result<Config, &'static str> {
        let matches = Command::new("downloader-cli")
            .version(CURRENT_VERSION)
            .arg(arg!(-m --manifest <String> "Path to manifest.json file or URL (e.g., http://localhost:8080/manifest.json)"))
            .arg(arg!(-p --provider <Provider> "Provider to use for downloads")
                .value_parser(clap::value_parser!(Provider))
                .default_value("cloudflare")
                .help("Available providers: cloudflare (Server #1), digitalocean (Server #2), none (Server #3 - Slowest)"))
            .arg(arg!(-v --verbose "Show verbose output including empty categories"))
            .arg(arg!(-y --yes "Automatically answer yes to all prompts and proceed with download"))
            .arg(arg!(-u --update "Check for application updates"))
            .get_matches();

        let manifest_location = if let Some(manifest_str) = matches.get_one::<String>("manifest") {
            Location::parse(manifest_str.to_string())?
        } else {
            Location::parse(DEFAULT_MANIFEST_URL.to_string())?
        };

        let manifest_provider = matches.get_one::<Provider>("provider").unwrap().clone();

        let verbose = matches.get_flag("verbose");
        let yes = matches.get_flag("yes");
        let check_updates = matches.get_flag("update");

        Ok(Config::new(
            manifest_location,
            manifest_provider,
            verbose,
            yes,
            check_updates,
        ))
    }
}
