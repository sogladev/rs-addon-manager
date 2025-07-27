#[cfg(target_os = "windows")]
pub fn wait_for_exit() {
    use std::io::{self, Write};
    println!("\nPress Enter to exit...");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}
