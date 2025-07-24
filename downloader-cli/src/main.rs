use std::env;
use std::error::Error;
use std::process::{self, Command};

use downloader_cli::Config;
use downloader_core::constants::{BIN_NAME_CLI, CURRENT_VERSION, REPO_NAME, REPO_OWNER};
use downloader_core::{banner, prompt, Progress};
use downloader_core::{Manifest, Transaction};

#[cfg(target_os = "windows")]
use std::io::Write;

fn main() {
    #[cfg(not(unix))]
    colored::control::set_virtual_terminal(true).unwrap();

    let config = Config::build_config().unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    // Self-update logic (must not run inside async context)
    if config.check_updates {
        let update_result = self_update::backends::github::Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(BIN_NAME_CLI)
            .show_download_progress(true)
            .current_version(CURRENT_VERSION)
            .build()
            .and_then(|u| u.update());

        match update_result {
            Ok(status) => {
                if status.updated() {
                    println!("Updated to version {}!", status.version());
                    println!("Restarting the application...");

                    // On Windows, the file may be locked briefly after replacement.
                    #[cfg(target_os = "windows")]
                    {
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
                            println!("Failed to restart: {e}");
                            println!("Please restart the application manually.");
                            process::exit(1);
                        }
                    }
                } else {
                    println!("No update available.");
                }
            }
            Err(e) => {
                println!("Failed to update: {e}");
            }
        }
    }

    // Now enter async context
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    if let Err(e) = rt.block_on(run(config)) {
        println!("Application error: {e}");
        process::exit(1);
    }
}

async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    banner::print_banner(&config.figure_text, &config.description);

    let base_path = std::env::current_dir().expect("Failed to get current directory");
    let manifest = Manifest::build(&config.manifest_location).await?;
    let transaction = Transaction::new(manifest, base_path);

    transaction.print(config.verbose);

    if transaction.has_pending_operations() {
        if !config.yes && !prompt::confirm("Is this ok")? {
            process::exit(1);
        }

        let progress_handler = |progress: &Progress| {
            progress.print();
            Ok(())
        };
        transaction
            .download(progress_handler, config.manifest_provider)
            .await?;
    }

    println!("\n{}", "-".repeat(100));
    println!("All files are up to date or successfully downloaded.");

    #[cfg(target_os = "windows")]
    {
        println!("\nPress Enter to exit...");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }

    Ok(())
}
