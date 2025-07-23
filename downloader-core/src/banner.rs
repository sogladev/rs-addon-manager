use figlet_rs::FIGfont;

/// Prints an ASCII art banner
///
/// This function loads a FIGlet font from an embedded resource file and converts
/// the provided `figure_text` (typically the application name) into ASCII art. It also
/// displays the given `description`, which can include project information such as a
/// description and repository URL, followed by a visual separator.
///
/// # Arguments
///
/// * `figure_text` - The text to be converted into ASCII art using the embedded FIGlet font.
/// * `description` - Additional information to display below the ASCII art banner, such as
///   a project description or repository URL.
///
/// # Panics
///
/// This function will panic if:
/// - The embedded font resource cannot be loaded or parsed
/// - The text conversion to ASCII art fails
///
/// The font resource is expected to be embedded at compile time and properly formatted.
pub fn print_banner(figure_text: &str, description: &str) {
    let slant_font_data = include_str!("../resources/slant.flf");
    let slant_font = FIGfont::from_content(slant_font_data).unwrap();
    let figure = slant_font.convert(figure_text);
    println!("{}", figure.unwrap());
    println!("{description}");
}
