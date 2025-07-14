use figlet_rs::FIGfont;

/// Prints an ASCII art banner
///
/// This function loads a FIGlet font from an embedded resource file and converts
/// the application name into ASCII art. It also displays project information
/// including a description and repository URL, followed by a visual separator.
///
/// # Panics
///
/// This function will panic if:
/// - The embedded font resource cannot be loaded or parsed
/// - The text conversion to ASCII art fails
///
/// The font resource is expected to be embedded at compile time and properly formatted.
pub fn print_banner() {
    let slant_font_data = include_str!("../resources/slant.flf");
    let slant_font = FIGfont::from_content(slant_font_data).unwrap();
    let figure = slant_font.convert("Project Epoch");
    print!("{}", figure.unwrap());
    println!("unofficial patch download utility - Sogladev");
    println!("Bugs or issues: https://github.com/sogladev/rs-game-launcher");
    println!("\n{}", "-".repeat(100));
}
