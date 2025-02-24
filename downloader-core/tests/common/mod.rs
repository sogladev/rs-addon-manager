use std::env::temp_dir;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;

pub struct TempFile {
    pub path: PathBuf,
}

impl TempFile {
    pub fn new(filename: &str, content: &str) -> Self {
        let mut path = temp_dir();
        path.push(filename);

        let mut file = File::create(&path).expect("Failed to create temp file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to temp file");

        TempFile { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        remove_file(&self.path).expect("Failed to delete temp file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempfile_creation() {
        let _temp = TempFile::new("test.txt", "Hello, world!");
    }
}
