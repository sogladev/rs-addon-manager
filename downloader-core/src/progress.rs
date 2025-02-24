use std::io::Write;
use std::time::Duration;

use humansize::{format_size, DECIMAL};

const MAX_FILENAME_LENGTH: usize = 20;
const PROGRESS_BAR_WIDTH: usize = 20;

#[derive(serde::Serialize)]
/// Represents the progress information for a file download or processing task.
pub struct Progress {
    /// The number of bytes processed for the current file.
    pub current: u64,
    /// The index of the current file being processed.
    pub file_index: usize,
    /// The total number of files to be processed.
    pub total_files: usize,
    /// The current processing speed in bytes per second.
    pub speed: f64,
    /// The total size of the current file in bytes.
    pub file_size: u64,
    /// The duration elapsed since the processing of the current file started.
    pub elapsed: Duration,
    /// The name of the current file.
    pub filename: String,
    /// The cumulative size of data downloaded across all files.
    pub total_size_downloaded: u64,
    /// The total amount of data remaining to be downloaded in bytes.
    pub total_amount_left: u64,
    /// The estimated time (in seconds) remaining to complete the download.
    pub expected_time_left: f64,
    /// The total download size of all files combined in bytes.
    pub total_download_size: i64,
}

impl Progress {
    fn truncate_filename(name: &str) -> String {
        if name.len() <= MAX_FILENAME_LENGTH {
            format!("{:width$}", name, width = MAX_FILENAME_LENGTH)
        } else {
            format!("{}...", &name[..MAX_FILENAME_LENGTH - 3])
        }
    }

    fn create_progress_bar(current: u64, total: u64) -> String {
        let progress = current as f64 / total as f64;
        let filled = (progress * PROGRESS_BAR_WIDTH as f64) as usize;
        format!(
            "[{}{}]",
            "-".repeat(filled),
            " ".repeat(PROGRESS_BAR_WIDTH - filled)
        )
    }

    pub fn print(&self) {
        let percent = (self.current as f64 / self.file_size as f64) * 100.0;
        let progress_bar = Self::create_progress_bar(self.current, self.file_size);
        let filename = Self::truncate_filename(&self.filename);
        let speed = format_size(self.speed as u64, DECIMAL);
        let total_files_width = self.total_files.to_string().len();
        let file_size = format_size(self.file_size, DECIMAL);
        let file_left = format_size(self.file_size.saturating_sub(self.current), DECIMAL);

        if self.current >= self.file_size {
            print!("\r\x1B[2K"); // Clear the line
            println!(
                "\r[{:>width$}/{}] {:<filename_width$} {} 100% (complete) | {:<8}/s | {} | ETA: {}",
                self.file_index,
                self.total_files,
                filename,
                progress_bar,
                speed,
                file_size,
                crate::format::eta_to_human_readable(self.expected_time_left),
                width = total_files_width,
                filename_width = MAX_FILENAME_LENGTH - 1
            );
        } else {
            print!("\r\x1B[2K"); // Clear the line
            print!(
                "\r[{:>width$}/{}] {:<filename_width$} {} {:5.1}% | {:<8}/s | {} | {} | ETA: {}",
                self.file_index,
                self.total_files,
                filename,
                progress_bar,
                percent,
                speed,
                file_size,
                file_left,
                crate::format::eta_to_human_readable(self.expected_time_left),
                width = total_files_width,
                filename_width = MAX_FILENAME_LENGTH - 1
            );
        }
        std::io::stdout().flush().unwrap(); // Ensure the output is flushed immediately
    }
}
