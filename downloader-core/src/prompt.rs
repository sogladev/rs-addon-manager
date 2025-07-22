use std::io::Write;

/// Prompt the user for confirmation
pub fn confirm(message: &str) -> std::io::Result<bool> {
    print!("{message} [Y/n]: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();
    Ok(trimmed.is_empty() || trimmed == "y")
}
