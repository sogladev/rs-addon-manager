// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{arg, command, Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Run the application in headless mode
    #[arg(long)]
    headless: bool,
    /// Suppress output in headless mode
    #[arg(long, short, requires = "headless")]
    quiet: bool,
}

fn main() {
    let args = Args::parse();

    if args.headless {
        addon_gui_lib::run_headless(args.quiet)
    } else {
        addon_gui_lib::run()
    }
}
