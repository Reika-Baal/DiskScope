mod format;
mod scanner;
mod statistics;

use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::path::{Path, PathBuf};

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

    #[arg(long, value_name = "DEPTH")]
    max_depth: Option<usize>,

    #[arg(long, value_name = "EXTENSION")]
    extension: Option<String>,

    #[arg(long, help = "Follow symbolic links while avoiding repeated paths")]
    follow_links: bool,
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

/// check whether a file matches the request
/// matching is case insensitive
fn matches_extension(path: &Path, extension: &str) -> bool {
    let requested_extension = extension.trim_start_matches('.');

    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(requested_extension))
}

fn main() {
    let cli = Cli::parse();

    // validate directory using scanner module
    if let Err(error) = scanner::validate_directory(&cli.directory) {
        eprintln!("{} {}", "Error:".bright_red().bold(), error);
        std::process::exit(1);
    }

    println!("{}", "====================================".bright_blue());
    println!(
        "{}",
        "             DiskScope              ".bright_cyan().bold()
    );
    println!("{}", "====================================".bright_blue());

    println!("A disk space analyser built in Rust.");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    println!("Welcome to DiskScope!");
    println!("Selected directory: {}", cli.directory.display());
    println!(
        "Symbolic links: {}",
        if cli.follow_links {
            "Following"
        } else {
            "Skipped"
        }
    );

    // read contents of selected directory
    let entries = match scanner::list_directory(&cli.directory) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("{} {}", "Error:".bright_red().bold(), error);
            std::process::exit(1);
        }
    };

    println!();
    println!("{}", "Directory contents:".bright_cyan().bold());

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
    println!("{}", "Files found recursively:".bright_cyan().bold());

    let scan_report = match scanner::scan_directory(&cli.directory, cli.max_depth, cli.follow_links)
    {
        Ok(report) => report,
        Err(error) => {
            eprintln!("{} {}", "Error:".bright_red().bold(), error);
            std::process::exit(1);
        }
    };

    // preserve the scanner statistics.
    let directories_scanned = scan_report.directories_scanned;
    let discovered_files = scan_report.files.len();

    let mut files = scan_report.files;
    let mut warnings = scan_report.warnings;

    // filter discover files if extension was provided
    if let Some(extension) = cli.extension.as_deref() {
        files.retain(|file| matches_extension(file, extension));
    }

    let matching_files = files.len();

    // Collect file sizes before sorting and displaying results.
    let mut file_data: Vec<(PathBuf, u64)> = Vec::new();
    let mut total_size: u64 = 0;

    for file in files {
        let size = match scanner::get_file_size(&file) {
            Ok(size) => size,
            Err(error) => {
                warnings.push(error);
                continue;
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
    let table_heading = format!("{:<width$} | {:>15}", "FILE", "SIZE", width = name_width);

    println!("{}", table_heading.bright_blue().bold());

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

    let total_row = format!(
        "{:<width$} | {:>15}",
        "TOTAL",
        format::format_size(total_size),
        width = name_width
    );

    println!("{}", total_row.bright_green().bold());

    println!();
    println!("Total files found: {}", file_data.len());

    // sort files by size large first
    let mut largest_files = file_data.clone();
    sort_files(&mut largest_files, SortBy::Size);

    println!();
    println!("{}", "Largest files (top 5):".bright_cyan().bold());

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

    // calculate storage usage grouped by file extension
    let extension_stats = statistics::extension_statistics(&file_data);

    println!();
    println!("{}", "====================================".bright_blue());
    println!(
        "{}",
        "         Extension Statistics       ".bright_cyan().bold()
    );
    println!("{}", "====================================".bright_blue());

    let extension_heading = format!(
        "{:<24} | {:>10} | {:>15}",
        "EXTENSION", "FILES", "TOTAL SIZE"
    );

    println!("{}", extension_heading.bright_blue().bold());

    println!(
        "{}-+-{}-+-{}",
        "-".repeat(24),
        "-".repeat(10),
        "-".repeat(15)
    );

    if extension_stats.is_empty() {
        println!("No files available for extension analysis.");
    } else {
        for (extension, stat) in &extension_stats {
            println!(
                "{:<24} | {:>10} | {:>15}",
                extension,
                stat.file_count,
                format::format_size(stat.total_bytes)
            );
        }
    }

    // show a consolidated summary of the scan
    println!();
    println!("{}", "====================================".bright_blue());
    println!(
        "{}",
        "           Scan Summary             ".bright_cyan().bold()
    );
    println!("{}", "====================================".bright_blue());

    println!("Directories scanned: {}", directories_scanned);
    println!("Files discovered: {}", discovered_files);
    println!("Files matching filters: {}", matching_files);
    println!("Files analysed: {}", file_data.len());

    println!("Total analysed size: {}", format::format_size(total_size));

    let warning_text = format!("Warnings: {}", warnings.len());

    if warnings.is_empty() {
        println!("{}", warning_text.bright_green());
    } else {
        println!("{}", warning_text.bright_yellow().bold());
    }

    if warnings.is_empty() {
        println!(
            "Scan status: {}",
            "Complete for selected scan options".bright_green().bold()
        );
    } else {
        println!(
            "Scan status: {}",
            "Partial - some files or directories were skipped"
                .bright_yellow()
                .bold()
        );
    }

    // show detailed warnings separately
    if !warnings.is_empty() {
        eprintln!();
        eprintln!(
            "{}",
            "Warnings encountered during scanning:"
                .bright_yellow()
                .bold()
        );

        for warning in &warnings {
            eprintln!("  {} {}", "Warning:".bright_yellow().bold(), warning);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SortBy, matches_extension, sort_files};
    use std::path::{Path, PathBuf};

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

    #[test]
    fn matches_file_extensions() {
        assert!(matches_extension(Path::new("report.pdf"), "pdf"));
        assert!(matches_extension(Path::new("image.png"), "png"));
        assert!(matches_extension(Path::new("main.rs"), "rs"));
    }

    #[test]
    fn matches_extensions_case_insensitively() {
        assert!(matches_extension(Path::new("REPORT.PDF"), "pdf"));
        assert!(matches_extension(Path::new("photo.PNG"), "PNG"));
        assert!(matches_extension(Path::new("document.pdf"), ".PDF"));
    }

    #[test]
    fn rejects_non_matching_extensions() {
        assert!(!matches_extension(Path::new("report.pdf"), "txt"));
        assert!(!matches_extension(Path::new("image.png"), "jpg"));
        assert!(!matches_extension(Path::new("README"), "txt"));
    }
}
