//! Define constants based on feature flags

pub const BIN_NAME_CLI: &str = "downloader-cli";
pub const REPO_OWNER: &str = "sogladev";
pub const REPO_NAME: &str = "rs-game-launcher";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "production")]
pub const IS_PRODUCTION: bool = true;
#[cfg(not(feature = "production"))]
pub const IS_PRODUCTION: bool = false;

pub fn default_version_label() -> String {
    if IS_PRODUCTION {
        format!("v{VERSION}")
    } else {
        format!("v{VERSION}-demo")
    }
}

pub fn default_manifest_url() -> &'static str {
    if IS_PRODUCTION {
        "https://updater.project-epoch.net/api/v2/manifest?environment=production"
    } else {
        "http://localhost:8080/manifest.json"
    }
}

pub fn default_figure_text() -> &'static str {
    if IS_PRODUCTION {
        "Project Epoch"
    } else {
        "Demo Launcher"
    }
}

pub fn default_description() -> String {
    if IS_PRODUCTION {
        format!(
            "unofficial patch download utility - Sogladev v{}\n\
            Bugs or issues: https://github.com/sogladev/rs-game-launcher\n\
            {VERSION}",
            "-".repeat(100)
        )
    } else {
        format!(
            "Demo version - For testing purposes only v{}-demo\n\
            Bugs or issues: https://github.com/sogladev/rs-game-launcher\n\
            {VERSION}",
            "-".repeat(100)
        )
    }
}
