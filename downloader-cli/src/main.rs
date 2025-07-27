use std::error::Error;
use std::process;

use clap::Parser;
use downloader_cli::{prompt, updater, Config};
use downloader_core::{banner, Progress};
use downloader_core::{Manifest, Transaction};

#[cfg(target_os = "windows")]
use std::io::Write;

fn main() {
    #[cfg(not(unix))]
    colored::control::set_virtual_terminal(true).unwrap();

    let config = Config::parse();

    if config.update {
        updater::self_update();
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
    let manifest = Manifest::build(&config.manifest).await?;
    let transaction = Transaction::new(manifest.clone(), base_path);

    transaction.print(config.verbose);

    if transaction.has_pending_operations() {
        let mut provider = config.provider;

        if !config.yes {
            match provider {
                Some(_) => {
                    if !prompt::confirm("Is this ok")? {
                        process::exit(1);
                    }
                }
                None => {
                    provider = prompt::select_provider(&manifest.files);
                }
            }
        }

        let progress_handler = |progress: &Progress| {
            progress.print();
            Ok(())
        };
        transaction
            .download(progress_handler, provider.unwrap_or_default())
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
