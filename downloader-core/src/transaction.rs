use std::error::Error;
use std::path::PathBuf;

use colored::Colorize;
use futures::StreamExt;
use humansize::BINARY;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use super::manifest::{Manifest, PatchFile, Provider};
use super::Progress;

#[derive(PartialEq, Clone)]
enum Status {
    Present,
    OutOfDate,
    Missing,
}

#[derive(PartialEq, Clone)]
enum OperationType {
    FileUpdate(PatchFile),
    FileRemoval(String),
}

#[derive(Clone)]
/// Represents a transaction operation involving file patching or removal.
///
/// This struct holds the details for an operation that either patches a file or removes a file.
///
/// # Fields
/// - `operation_type`: The type of operation (file update or removal).
/// - `size`: The current size of the file, represented as a 64-bit signed integer.
/// - `status`: The status of the file operation.
struct FileOperation {
    operation_type: OperationType,
    size: i64,
    status: Status,
}

impl FileOperation {
    /// Process the manifest and return a list of file operations
    fn process(manifest: &Manifest, base_path: &std::path::Path) -> Vec<FileOperation> {
        let mut operations = Vec::new();

        // Process file updates
        for file in &manifest.files {
            let full_path = base_path.join(&file.path);
            if !full_path.exists() {
                operations.push(FileOperation {
                    status: Status::Missing,
                    operation_type: OperationType::FileUpdate(file.clone()),
                    size: 0,
                });
                continue;
            }

            match std::fs::read(&full_path) {
                Ok(contents) => {
                    let digest = md5::compute(contents);
                    let digest_str = format!("{digest:x}");
                    let new_size: i64 = std::fs::metadata(&full_path)
                        .unwrap_or_else(|_| {
                            panic!("Failed to read metadata for file: {:?}", &full_path)
                        })
                        .len()
                        .try_into()
                        .unwrap();

                    operations.push(FileOperation {
                        status: if digest_str == file.hash {
                            Status::Present
                        } else {
                            Status::OutOfDate
                        },
                        operation_type: OperationType::FileUpdate(file.clone()),
                        size: new_size,
                    });
                }
                Err(e) => {
                    panic!("Failed to read file {}: {}", full_path.display(), e);
                }
            }
        }

        // Process file removals
        if let Some(removals) = &manifest.removals {
            for removal_path in removals {
                let full_path = base_path.join(removal_path);
                let size = if full_path.exists() {
                    match std::fs::metadata(&full_path) {
                        Ok(metadata) => metadata.len() as i64,
                        Err(e) => {
                            eprintln!("Warning: Failed to read metadata for file to be removed {}: {}. Using size 0.", full_path.display(), e);
                            0
                        }
                    }
                } else {
                    0
                };

                operations.push(FileOperation {
                    status: if full_path.exists() {
                        Status::OutOfDate
                    } else {
                        Status::Present
                    },
                    operation_type: OperationType::FileRemoval(removal_path.clone()),
                    size,
                });
            }
        }

        operations
    }

    /// Get the file path for this operation
    fn get_path(&self) -> &str {
        match &self.operation_type {
            OperationType::FileUpdate(patch_file) => &patch_file.path,
            OperationType::FileRemoval(path) => path,
        }
    }

    /// Get the patch file if this is a file update operation
    fn get_patch_file(&self) -> Option<&PatchFile> {
        match &self.operation_type {
            OperationType::FileUpdate(patch_file) => Some(patch_file),
            OperationType::FileRemoval(_) => None,
        }
    }

    /// Check if this is a file update operation
    fn is_file_update(&self) -> bool {
        matches!(self.operation_type, OperationType::FileUpdate(_))
    }

    /// Check if this is a file removal operation
    fn is_file_removal(&self) -> bool {
        matches!(self.operation_type, OperationType::FileRemoval(_))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReport {
    pub path: String,
    pub current_size: Option<i64>, // None if file is missing
    pub new_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionReport {
    pub version: String,
    pub uid: String,
    pub up_to_date_files: Vec<FileReport>,
    pub outdated_files: Vec<FileReport>,
    pub missing_files: Vec<FileReport>,
    pub removed_files: Vec<FileReport>,
    pub total_download_size: u64,
    pub disk_space_change: i64,
    pub base_path: PathBuf,
}

#[derive(Clone)]
pub struct Transaction {
    operations: Vec<FileOperation>,
    manifest_version: String,
    manifest_uid: String,
    pub base_path: PathBuf,
}

impl Transaction {
    pub fn new(manifest: Manifest, base_path: PathBuf) -> Self {
        let operations = FileOperation::process(&manifest, &base_path);
        Transaction {
            operations,
            manifest_version: manifest.version,
            manifest_uid: manifest.uuid,
            base_path,
        }
    }

    pub fn generate_report(&self) -> TransactionReport {
        TransactionReport {
            version: self.manifest_version.clone(),
            uid: self.manifest_uid.clone(),
            up_to_date_files: self
                .up_to_date()
                .iter()
                .filter_map(|op| {
                    op.get_patch_file().map(|patch_file| FileReport {
                        path: patch_file.path.clone(),
                        current_size: Some(op.size),
                        new_size: patch_file.size,
                    })
                })
                .collect(),
            outdated_files: self
                .outdated()
                .iter()
                .filter_map(|op| {
                    op.get_patch_file().map(|patch_file| FileReport {
                        path: patch_file.path.clone(),
                        current_size: Some(op.size),
                        new_size: patch_file.size,
                    })
                })
                .collect(),
            missing_files: self
                .missing()
                .iter()
                .filter_map(|op| {
                    op.get_patch_file().map(|patch_file| FileReport {
                        path: patch_file.path.clone(),
                        current_size: None,
                        new_size: patch_file.size,
                    })
                })
                .collect(),
            removed_files: self
                .removals()
                .iter()
                .map(|op| FileReport {
                    path: op.get_path().to_string(),
                    current_size: Some(op.size),
                    new_size: 0,
                })
                .collect(),
            total_download_size: self.total_download_size() as u64,
            disk_space_change: self.disk_space_change(),
            base_path: self.base_path.clone(),
        }
    }

    pub fn print(&self, verbose: bool) {
        let report = self.generate_report();
        println!("\nManifest Overview:");
        println!(" Version: {}", report.version);
        println!(" UID: {}", report.uid);
        println!(" Base path: {}", report.base_path.display());

        if !report.up_to_date_files.is_empty() || verbose {
            println!("\n {}", "Up-to-date files:".green());
        }
        for file in &report.up_to_date_files {
            println!(
                "  {} (Size: {})",
                file.path.green(),
                humansize::format_size(file.new_size as u64, BINARY)
            );
        }

        if !report.outdated_files.is_empty() || verbose {
            println!("\n {}", "Outdated files (will be updated):".yellow());
        }
        for file in &report.outdated_files {
            println!(
                "  {} (Current Size: {}, New Size: {})",
                file.path.yellow(),
                humansize::format_size(file.current_size.unwrap() as u64, BINARY),
                humansize::format_size(file.new_size as u64, BINARY)
            );
        }

        if !report.missing_files.is_empty() || verbose {
            println!("\n {}", "Missing files (will be downloaded):".red());
        }
        for file in &report.missing_files {
            println!(
                "  {} (New Size: {})",
                file.path.red(),
                humansize::format_size(file.new_size as u64, BINARY)
            );
        }

        if !report.removed_files.is_empty() || verbose {
            println!("\n {}", "Files to be removed:".magenta());
        }
        for file in &report.removed_files {
            println!(
                "  {} (Current Size: {})",
                file.path.magenta(),
                humansize::format_size(file.current_size.unwrap_or(0) as u64, BINARY)
            );
        }

        if self.has_pending_operations() {
            println!("\nTransaction Summary:");
            println!(" Installing/Updating: {} files", self.pending_count());
            println!(
                "\nTotal size of inbound files is {}. Need to download {}.",
                humansize::format_size(report.total_download_size, BINARY),
                humansize::format_size(report.total_download_size, BINARY)
            );

            let disk_space_change = report.disk_space_change;
            if disk_space_change > 0 {
                println!(
                    "After this operation, {} of additional disk space will be used.",
                    humansize::format_size(disk_space_change as u64, BINARY)
                );
            } else {
                println!(
                    "After this operation, {} of disk space will be freed.",
                    humansize::format_size(disk_space_change.unsigned_abs(), BINARY)
                );
            }
        }
    }

    fn up_to_date(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status == Status::Present)
            .collect()
    }

    fn outdated(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status == Status::OutOfDate)
            .collect()
    }

    fn pending(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status != Status::Present)
            .collect()
    }

    fn missing(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.status == Status::Missing)
            .collect()
    }

    fn removals(&self) -> Vec<&FileOperation> {
        self.operations
            .iter()
            .filter(|op| op.is_file_removal())
            .collect()
    }

    pub fn pending_count(&self) -> usize {
        self.operations
            .iter()
            .filter(|x| x.status != Status::Present)
            .count()
    }

    pub fn has_pending_operations(&self) -> bool {
        self.pending_count() > 0
    }

    fn total_download_size(&self) -> i64 {
        let total = self
            .operations
            .iter()
            .filter(|x| x.status != Status::Present && x.is_file_update())
            .filter_map(|x| x.get_patch_file())
            .map(|x| x.size)
            .sum();
        assert!(
            total >= 0,
            "Total download size must be non-negative, but found {total}."
        );
        total
    }

    fn disk_space_change(&self) -> i64 {
        self.operations
            .iter()
            .filter(|x| x.status != Status::Present)
            .map(|x| match &x.operation_type {
                OperationType::FileUpdate(patch_file) => patch_file.size - x.size,
                OperationType::FileRemoval(_) => -x.size, // Removing a file frees up space
            })
            .sum()
    }

    pub async fn download<F>(
        &self,
        progress_handler: F,
        provider: Provider,
    ) -> Result<(), Box<dyn Error>>
    where
        F: Fn(&Progress) -> Result<(), Box<dyn Error>> + Send + 'static,
    {
        let http_client = reqwest::Client::new();
        let mut total_size_downloaded = 0;
        let total_download_size = self.total_download_size();

        for (idx, op) in self.pending().iter().enumerate() {
            match &op.operation_type {
                OperationType::FileUpdate(patch_file) => {
                    // Handle file downloads/updates
                    let dest_path = self.base_path.join(&patch_file.path);

                    // Create parent directories if they don't exist
                    if let Some(dir) = dest_path.parent() {
                        tokio::fs::create_dir_all(dir).await?;
                    }

                    // Get URL for the specified provider
                    let url = patch_file.get_url(&provider).ok_or_else(|| {
                        format!(
                            "No URL found for provider {:?} for file {}",
                            provider, patch_file.path
                        )
                    })?;

                    let response = http_client.get(url).send().await?;
                    if !response.status().is_success() {
                        eprintln!("Failed to download {}: {}", url, response.status());
                        continue;
                    }

                    let file_size = patch_file.size;
                    let mut file = tokio::fs::File::create(dest_path.clone()).await?;
                    let start = std::time::Instant::now();
                    let mut downloaded: u64 = 0;

                    let mut stream = response.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.map_err(|e| e.to_string())?;
                        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
                        downloaded += chunk.len() as u64;
                        total_size_downloaded += chunk.len() as u64;

                        // Handle potential underflow
                        let total_amount_left =
                            (total_download_size as u64).saturating_sub(total_size_downloaded);

                        // Compute download speed and expected time left
                        let speed = downloaded as f64 / start.elapsed().as_secs_f64();
                        let expected_time_left = if speed > 0.0 {
                            // Compute remaining time and cap at, say, 24 hours (86400 s).
                            (total_amount_left as f64 / speed).min(86400.0)
                        } else {
                            0.0
                        };

                        let progress = Progress {
                            current: downloaded,
                            file_index: idx + 1,
                            total_files: self.pending_count(),
                            speed,
                            file_size: file_size.try_into().unwrap(),
                            elapsed: start.elapsed(),
                            filename: dest_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),

                            total_size_downloaded,
                            total_amount_left,
                            expected_time_left,
                            total_download_size,
                        };

                        progress_handler(&progress)?;
                    }
                }

                OperationType::FileRemoval(path) => {
                    // Handle file removals
                    let dest_path = self.base_path.join(path);
                    if dest_path.exists() {
                        tokio::fs::remove_file(&dest_path).await.map_err(|e| {
                            format!("Failed to remove file {}: {}", dest_path.display(), e)
                        })?;
                        println!("Removed file: {}", dest_path.display());
                    }
                }
            }
        }
        Ok(())
    }
}
