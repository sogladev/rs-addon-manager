/// Verifies the integrity of a game installation by checking for required files and directories.
///
/// This function checks if all required game files and directories exist in the specified game directory.
/// It looks for specific files like DLLs and MPQ files, as well as essential directories.
///
/// # Arguments
/// * `game_dir` - A Path reference pointing to the root directory of the game installation
///
/// # Returns
/// * `Result<bool, std::io::Error>` - Returns Ok(true) if all required files and directories exist,
///   Ok(false) if any required file or directory is missing, or Err if an IO error occurs
///
/// # Examples
/// ```
/// use rs_manifest_patcher::game::verify_game_integrity;
/// let game_path = std::path::Path::new("C:/Games/WoW");
/// match verify_game_integrity(game_path) {
///     Ok(true) => println!("Game files verified successfully"),
///     Ok(false) => println!("Game files are missing"),
///     Err(e) => println!("Error checking game files: {}", e),
/// }
/// ```
#[allow(dead_code)]
pub fn verify_game_integrity(game_dir: &std::path::Path) -> Result<bool, std::io::Error> {
    let required_files = ["Battle.net.dll", "Data/lichking.MPQ", "Data/patch-3.MPQ"];

    let required_dirs = ["Data"];

    // Check required directories
    for dir in required_dirs.iter() {
        let dir_path = game_dir.join(dir);
        if !dir_path.is_dir() {
            println!("Missing required directory: {dir}");
            return Ok(false);
        }
    }

    // Check required files
    for file in required_files.iter() {
        let file_path = game_dir.join(file);
        if !file_path.is_file() {
            println!("Missing required file: {file}");
            return Ok(false);
        }
    }

    Ok(true)
}

#[allow(dead_code)]
pub fn launch(client_directory: &std::path::Path, executable_name: String) -> std::io::Result<()> {
    // Clear the cache directory
    let cache_path = client_directory.join("Cache");
        if cache_path.exists() {
        std::fs::remove_dir_all(&cache_path)?;
    }

    let executable_path = client_directory.join(executable_name);
    match std::env::consts::OS {
        "linux" => {
            let wine_prefix = std::env::var("WINEPREFIX")
                .unwrap_or_else(|_| client_directory.join(".wine").to_string_lossy().to_string());
            let command = format!(
                "WINEPREFIX=\"{}\" wine \"{}\"",
                wine_prefix,
                executable_path.display()
            );
            std::process::Command::new("setsid")
                .arg("sh")
                .arg("-c")
                .arg(command)
                .spawn()?;
        }
        "windows" => {
            std::process::Command::new(executable_path).spawn()?;
        }
        _ => {
            return Err(std::io::Error::other("Unsupported platform".to_string()));
        }
    }
    Ok(())
}
