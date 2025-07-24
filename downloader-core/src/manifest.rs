use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Provider {
    Cloudflare,
    #[serde(rename = "digitalocean")]
    #[cfg_attr(feature = "cli", clap(name = "digitalocean"))]
    DigitalOcean,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
/// Represents a patch file with its associated metadata.
///
/// # Fields
///
/// - `path` - A string containing the file path where the patch file is located.
/// - `hash` - A string representing the checksum or hash of the file, used for integrity verification.
/// - `size` - A 64-bit integer indicating the file size in bytes.
/// * `custom` - A boolean flag that indicates if the patch file is custom.
/// * `urls` - A map of provider names to their corresponding URLs.
pub struct PatchFile {
    pub path: String,
    pub hash: String,
    pub size: i64,
    pub custom: bool,
    pub urls: HashMap<Provider, String>,
}

impl PatchFile {
    /// Get URL for a specific provider, falling back to "none" if not found
    pub fn get_url(&self, provider: &Provider) -> Option<&String> {
        self.urls
            .get(provider)
            .or_else(|| self.urls.get(&Provider::None))
    }

    /// Get all available providers for this file
    pub fn available_providers(&self) -> Vec<&Provider> {
        self.urls.keys().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
/// Represents a manifest configuration that includes version information
/// and a collection of patch files.
///
/// # Fields
///
/// - `version`: A String representing the manifest's version.
/// - `uuid`: A String representing the unique identifier for the manifest.
/// - `files`: A vector of `PatchFile` items, each corresponding to a file that is
/// - `removals`: An optional vector of strings representing file paths that should be removed,
///   subject to patching.
pub struct Manifest {
    pub version: String,
    #[serde(rename = "Uid")]
    pub uuid: String,
    pub files: Vec<PatchFile>,
    pub removals: Option<Vec<String>>,
}

impl Manifest {
    /// Create a new Manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, Box<dyn Error>> {
        let mut manifest: Manifest = serde_json::from_str(json)?;

        // Convert paths from Windows to Unix format
        manifest
            .files
            .iter_mut()
            .for_each(|file| file.path = file.path.replace("\\", "/"));

        Ok(manifest)
    }

    /// Load manifest from a file
    pub fn from_file(file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let contents = std::fs::read_to_string(file_path)?;
        Self::from_json(&contents)
    }

    /// Build manifest from a location (URL or file)
    pub async fn build(url: &Url) -> Result<Self, Box<dyn Error>> {
        let response = reqwest::get(url.as_str()).await?;
        let contents = response.text().await?;
        Self::from_json(&contents)
    }
}
