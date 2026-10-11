mod scanner;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "diskscope",
    version,
    about = "A disk space analyser built in Rust.",
    long_about = None
)]
struct Cli {
    #[arg(value_name = "DIRECTORY", default_value = ".")]
    directory: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    // validate directory using scanner module.
    if let Err(error) = scanner::validate_directory(&cli.directory) {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }

    println!("====================================");
    println!("             DiskScope              ");
    println!("====================================");

    println!("A disk space analyser built in Rust.");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    println!("Welcome to DiskScope!");
    println!("Selected directory: {}", cli.directory.display());
    println!("Directory scanning coming soon...");
}
