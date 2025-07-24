//! Define constants based on feature flags

#[cfg(feature = "production")]
pub const DEFAULT_MANIFEST_URL: &str =
    "https://updater.project-epoch.net/api/v2/manifest?environment=production";
#[cfg(feature = "production")]
pub const DEFAULT_FIGURE_TEXT: &str = "Project Epoch";
#[cfg(feature = "production")]
pub const DEFAULT_DESCRIPTION: &str =
    "unofficial patch download utility - Sogladev\n\
    Bugs or issues: https://github.com/sogladev/rs-game-launcher\n\
    ----------------------------------------------------------------------------------------------------";

#[cfg(not(feature = "production"))]
pub const DEFAULT_MANIFEST_URL: &str = "http://localhost:8080/manifest.json";
#[cfg(not(feature = "production"))]
pub const DEFAULT_FIGURE_TEXT: &str = "Demo Launcher";
#[cfg(not(feature = "production"))]
pub const DEFAULT_DESCRIPTION: &str =
    "Demo version - For testing purposes only\n\
    Bugs or issues: https://github.com/sogladev/rs-game-launcher\n\
    ----------------------------------------------------------------------------------------------------";

pub const BIN_NAME_CLI: &str = "downloader-cli";
pub const REPO_OWNER: &str = "sogladev";
pub const REPO_NAME: &str = "rs-game-launcher";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
