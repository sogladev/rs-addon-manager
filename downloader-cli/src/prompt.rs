use downloader_core::Provider;
use std::collections::HashSet;
use std::io::{self, Write};

/// Prompt the user for confirmation
pub fn confirm(message: &str) -> std::io::Result<bool> {
    print!("{message} [Y/n]: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();
    Ok(trimmed.is_empty() || trimmed == "y")
}

/// Prompt the user to select a provider from the manifest files
pub fn select_provider(manifest_files: &[downloader_core::PatchFile]) -> Option<Provider> {
    let mut providers = HashSet::new();
    for file in manifest_files {
        for provider in file.urls.keys() {
            providers.insert(*provider);
        }
    }
    let mut providers: Vec<_> = providers.into_iter().collect();
    providers.sort_by_key(|p| format!("{p:?}"));

    let default_provider = Provider::default();

    println!("Select a provider:");
    for (i, p) in providers.iter().enumerate() {
        if p == &default_provider {
            if p == &Provider::None {
                println!("  {}: {:?} (Default, Slow)", i + 1, p);
            } else {
                println!("  {}: {:?} (Default)", i + 1, p);
            }
        } else if p == &Provider::None {
            println!("  {}: {:?} (Slow)", i + 1, p);
        } else {
            println!("  {}: {:?}", i + 1, p);
        }
    }
    print!(
        "Enter number [1-{}] (Default: {:?}): ",
        providers.len(),
        default_provider
    );
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;

    // Find index of default provider in sorted list
    let default_idx = providers
        .iter()
        .position(|p| p == &default_provider)
        .unwrap_or(0)
        + 1;

    let choice = input.trim().parse::<usize>().unwrap_or(default_idx);
    let idx = if choice == 0 || choice > providers.len() {
        default_idx - 1
    } else {
        choice - 1
    };
    Some(providers[idx])
}
