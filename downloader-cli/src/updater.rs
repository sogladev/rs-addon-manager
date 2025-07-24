use std::{env, process::{self, Command}};

use downloader_core::constants::{BIN_NAME_CLI, CURRENT_VERSION, REPO_NAME, REPO_OWNER};

pub fn self_update() {
    let update_result = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME_CLI)
        .show_download_progress(true)
        .current_version(CURRENT_VERSION)
        .build()
        .and_then(|u| u.update());

    match update_result {
        Ok(status) if status.updated() => {
            println!("Updated to version {}!", status.version());
            println!("Restarting the application...");

            #[cfg(target_os = "windows")]
            {
                // On Windows, the file may be locked briefly after replacement.
                println!("Waiting a moment to ensure update is complete...");
                std::thread::sleep(std::time::Duration::from_millis(1200));
            }

            let args: Vec<String> = env::args().collect();
            match Command::new(&args[0]).args(&args[1..]).spawn() {
                Ok(_) => {
                    println!("Launched new version, exiting old process.");
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("Failed to restart: {e}");
                    eprintln!("Please restart the application manually.");
                    process::exit(1);
                }
            }
        }
        Ok(_) => println!("No update available."),
        Err(e) => eprintln!("Failed to update: {e}"),
    }
}