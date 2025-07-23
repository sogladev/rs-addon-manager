use std::error::Error;
use std::process;

use downloader_core::{banner, prompt, Progress};
use downloader_core::{Config, Manifest, Transaction};

#[cfg(target_os = "windows")]
use std::io::Write;

#[tokio::main]
async fn main() {
    #[cfg(not(unix))]
    colored::control::set_virtual_terminal(true).unwrap();

    let config = Config::build().unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config).await {
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
        if !prompt::confirm("Is this ok")? {
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
