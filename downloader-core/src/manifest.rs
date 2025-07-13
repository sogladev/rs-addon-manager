use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone)]
pub enum Location {
    Url(Url),
    /// Wraps a [std::path::PathBuf] representing a file system path.
    ///
    /// This field stores an independently owned and mutable file system path.
    /// It leverages the platform-specific features of [std::path::PathBuf]
    /// to provide a reliable method for handling file paths regardless of the operating system.
    FilePath(PathBuf),
}

impl Location {
    /// Parse a manifest location string into a `Location` enum
    pub fn parse(manifest_str: String) -> Result<Self, &'static str> {
        if let Ok(parsed_url) = Url::parse(&manifest_str) {
            // A relative URL string should return an error
            if parsed_url.cannot_be_a_base() {
                return Err("URL is incomplete");
            }
            // Check if the URL has a valid scheme
            if parsed_url.scheme() == "http" || parsed_url.scheme() == "https" {
                return Ok(Location::Url(parsed_url));
            }
        }

        let path = PathBuf::from(&manifest_str);
        if path.exists() && std::fs::File::open(&path).is_ok() {
            return Ok(Location::FilePath(path));
        }

        Err(
            "Manifest location must be a valid URL (e.g., http://localhost:8080/manifest.json) \
            or a readable file path",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Cloudflare,
    #[serde(rename = "digitalocean")]
    DigitalOcean,
    None,
    #[serde(untagged)]
    Other(String),
}

impl Provider {
    /// Get the provider key as used in JSON and CLI
    pub fn key(&self) -> &str {
        match self {
            Provider::Cloudflare => "cloudflare",
            Provider::DigitalOcean => "digitalocean",
            Provider::None => "none",
            Provider::Other(name) => name,
        }
    }

    /// Get all known provider keys for CLI validation
    pub fn known_keys() -> Vec<&'static str> {
        vec!["cloudflare", "digitalocean", "none"]
    }

    /// Get the display name for UI purposes
    pub fn display_name(&self) -> &str {
        match self {
            Provider::Cloudflare => "Server #1",
            Provider::DigitalOcean => "Server #2",
            Provider::None => "Server #3 (Slowest)",
            Provider::Other(name) => name,
        }
    }
}

impl FromStr for Provider {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "cloudflare" => Provider::Cloudflare,
            "digitalocean" => Provider::DigitalOcean,
            "none" => Provider::None,
            other => Provider::Other(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
/// - `uid`: A String representing the unique identifier for the manifest.
/// - `files`: A vector of `PatchFile` items, each corresponding to a file that is
/// - `removals`: An optional vector of strings representing file paths that should be removed,
///   subject to patching.
pub struct Manifest {
    pub version: String,
    pub uid: String,
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
    pub async fn build(location: &Location) -> Result<Self, Box<dyn Error>> {
        match location {
            Location::Url(url) => {
                let response = reqwest::get(url.as_str()).await?;
                let contents = response.text().await?;
                Self::from_json(&contents)
            }
            Location::FilePath(file_path) => Self::from_file(file_path),
        }
    }
}
