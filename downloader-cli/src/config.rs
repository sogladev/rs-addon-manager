use clap::Parser;
use downloader_core::{
    constants::{default_description, default_figure_text, default_manifest_url, VERSION},
    manifest::Provider,
};
use url::Url;

#[derive(Debug, Parser)]
#[command(name = "downloader-cli", version = VERSION, about = "Rust-based patch downloader")]
pub struct Config {
    /// Path to manifest.json file or URL (e.g., http://localhost:8080/manifest.json)
    #[arg(short, long, default_value = default_manifest_url())]
    pub manifest: Url,

    /// Provider to use for downloads. If not specified, you will be prompted interactively.
    #[arg(
        short,
        long,
        value_enum,
        help = "Available providers: cloudflare (Server #1), digitalocean (Server #2), none (Server #3 - Slowest). If not specified, you will be prompted interactively."
    )]
    pub provider: Option<Provider>,

    /// Show verbose output including empty categories
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Automatically answer yes to all prompts and proceed with download
    #[arg(short, long, default_value_t = false)]
    pub yes: bool,

    /// Check for application updates
    #[arg(short = 'u', long, default_value_t = false)]
    pub update: bool,

    /// ASCII art figure text (internal use)
    #[arg(skip = default_figure_text())]
    pub figure_text: String,

    /// Description (internal use)
    #[arg(skip = default_description())]
    pub description: String,
}
