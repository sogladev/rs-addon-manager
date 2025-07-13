/// Prints the banner using a FIGlet font.
///
/// This function loads the ASCII art font from an external resource file (slant.flf)
/// using `include_str!`, and creates a FIGlet font instance with `FIGfont::from_content`.
/// It then converts the text "Project Epoch" into ASCII art and prints it to the console.
/// Additionally, the function prints a tagline indicating that this is an unofficial patch download
/// utility by Sogladev, along with a URL for reporting bugs or issues. Lastly, it prints a line
/// of dashes as a visual separator.
///
/// # Panics
///
/// This function will panic if:
/// - The font data cannot be loaded from the specified resource file.
/// - The FIGlet font cannot be created or the conversion of the text to ASCII art fails.
///
/// Therefore, it is assumed that the required font resource exists and is correctly formatted.
use figlet_rs::FIGfont;

pub fn print_banner() {
    let slant_font_data = include_str!("../resources/slant.flf");
    let slant_font = FIGfont::from_content(slant_font_data).unwrap();
    let figure = slant_font.convert("Project Epoch");
    print!("{}", figure.unwrap());
    println!("unofficial patch download utility - Sogladev");
    println!("Bugs or issues: https://github.com/sogladev/rs_manifest_patcher/");
    println!("\n{}", "-".repeat(100));
}
