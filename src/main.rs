mod format;
mod scanner;

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
enum SortBy {
    Name,
    Size,
}

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

    #[arg(long, value_enum, default_value_t = SortBy::Name)]
    sort: SortBy,
}

fn sort_files(files: &mut [(PathBuf, u64)], sort_by: SortBy) {
    match sort_by {
        SortBy::Name => {
            files.sort_by(|a, b| a.0.cmp(&b.0));
        }

        SortBy::Size => {
            files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // validate directory using scanner module
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

    // Collect file sizes before sorting and displaying results.
    let mut file_data: Vec<(PathBuf, u64)> = Vec::new();
    let mut total_size: u64 = 0;

    for file in files {
        let size = match scanner::get_file_size(&file) {
            Ok(size) => size,
            Err(error) => {
                eprintln!("Error: {}", error);
                std::process::exit(1);
            }
        };

        total_size = match total_size.checked_add(size) {
            Some(total) => total,
            None => {
                eprintln!("Error: Total directory size exceeds the supported limit.");
                std::process::exit(1);
            }
        };

        file_data.push((file, size));
    }

    // sorting method chosen
    sort_files(&mut file_data, cli.sort);

    let name_width = file_data
        .iter()
        .map(|(path, _)| path.display().to_string().chars().count())
        .max()
        .unwrap_or(4)
        .max(4);

    // show the table header
    println!();
    println!("{:<width$} | {:>15}", "FILE", "SIZE", width = name_width);

    // show the summary underneath the table
    println!("{}-+-{}", "-".repeat(name_width), "-".repeat(15));

    if file_data.is_empty() {
        println!("No files found.");
    } else {
        for (path, size) in &file_data {
            println!(
                "{:<width$} | {:>15}",
                path.display(),
                format::format_size(*size),
                width = name_width
            );
        }
    }

    // show the summary underneath the table
    println!("{}-+-{}", "-".repeat(name_width), "-".repeat(15));

    println!(
        "{:<width$} | {:>15}",
        "TOTAL",
        format::format_size(total_size),
        width = name_width
    );

    println!();
    println!("Total files found: {}", file_data.len());

    // sort files by size large first
    let mut largest_files = file_data.clone();
    sort_files(&mut largest_files, SortBy::Size);

    println!();
    println!("Largest files (top 5):");

    if largest_files.is_empty() {
        println!("  No files found.");
    } else {
        for (index, (path, size)) in largest_files.iter().take(5).enumerate() {
            println!(
                "  {}. {} ({})",
                index + 1,
                path.display(),
                format::format_size(*size)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SortBy, sort_files};
    use std::path::PathBuf;

    #[test]
    fn sorts_files_by_name() {
        let mut files = vec![
            (PathBuf::from("z.txt"), 100),
            (PathBuf::from("a.txt"), 300),
            (PathBuf::from("m.txt"), 200),
        ];

        sort_files(&mut files, SortBy::Name);

        assert_eq!(files[0].0, PathBuf::from("a.txt"));
        assert_eq!(files[1].0, PathBuf::from("m.txt"));
        assert_eq!(files[2].0, PathBuf::from("z.txt"));
    }

    #[test]
    fn sorts_files_by_size_descending() {
        let mut files = vec![
            (PathBuf::from("small.txt"), 100),
            (PathBuf::from("large.txt"), 500),
            (PathBuf::from("medium.txt"), 250),
        ];

        sort_files(&mut files, SortBy::Size);

        assert_eq!(files[0].1, 500);
        assert_eq!(files[1].1, 250);
        assert_eq!(files[2].1, 100);
    }
}
