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

    // read contents of selected directory
    let entries = match scanner::list_directory(&cli.directory) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    };

    println!();
    println!("Directory contents:");

    if entries.is_empty() {
        println!("  This directory is empty.");
    } else {
        for entry in &entries {
            println!("  {}", entry.file_name().to_string_lossy());
        }
    }

    println!();
    println!("Total entries: {}", entries.len());

    println!();
    println!("Files found recursively:");

    let files = match scanner::scan_directory(&cli.directory) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    };

    if files.is_empty() {
        println!("  No files found.");
    } else {
        for file in &files {
            let size = match scanner::get_file_size(file) {
                Ok(size) => size,
                Err(error) => {
                    eprintln!("Error: {}", error);
                    std::process::exit(1);
                }
            };

            println!("  {} ({} bytes)", file.display(), size);
        }
    }

    println!();
    println!("Total files found: {}", files.len());
}
