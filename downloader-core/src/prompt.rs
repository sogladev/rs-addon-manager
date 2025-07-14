use std::io::Write;

/// Prompt the user for confirmation [y/N]
pub fn confirm(message: &str) -> std::io::Result<bool> {
    print!("{message} [y/N]: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}
