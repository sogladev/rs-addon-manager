use std::io::{self, Write};

/// Prompt the user for confirmation [y/N]
pub fn confirm(message: &str) -> io::Result<bool> {
    print!("{} [y/N]: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}
