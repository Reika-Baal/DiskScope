use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// validates specified path exists & is a directory.
pub fn validate_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            format!("Path does not exist: {}", path.display())
        } else {
            format!("Cannot access '{}': {}", path.display(), error)
        }
    })?;

    if !metadata.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    Ok(())
}

/// reads & returns the entries within a specified directory.
pub fn list_directory(path: &Path) -> Result<Vec<fs::DirEntry>, String> {
    let directory = fs::read_dir(path)
        .map_err(|error| format!("Cannot read directory '{}': {}", path.display(), error))?;

    let mut entries = Vec::new();

    for entry in directory {
        let entry = entry
            .map_err(|error| format!("Cannot read entry in '{}': {}", path.display(), error))?;

        entries.push(entry);
    }

    // Sort entries alphabetically for consistent output.
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());

    Ok(entries)
}

#[derive(Debug)]
pub struct ScanReport {
    pub files: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub directories_scanned: usize,
}

/// recursively scans a directory & returns all regular file paths
/// optional maximum depth limit how deeply subdirectories are explored
pub fn scan_directory(
    path: &Path,
    max_depth: Option<usize>,
    follow_links: bool,
    mut on_progress: impl FnMut(usize, usize),
) -> Result<ScanReport, String> {
    let mut report = ScanReport {
        files: Vec::new(),
        warnings: Vec::new(),
        directories_scanned: 0,
    };

    // track directories and files to prevent repeated visits when following symbolic links
    let mut visited_dirs = HashSet::new();
    let mut visited_files = HashSet::new();

    scan_recursive(
        path,
        &mut report,
        0,
        max_depth,
        follow_links,
        &mut visited_dirs,
        &mut visited_files,
        &mut on_progress,
    )?;

    report.files.sort();

    // report final discovery counts
    on_progress(report.directories_scanned, report.files.len());

    Ok(report)
}

/// visits a directory & its subdirectories
fn scan_recursive(
    path: &Path,
    report: &mut ScanReport,
    current_depth: usize,
    max_depth: Option<usize>,
    follow_links: bool,
    visited_dirs: &mut HashSet<PathBuf>,
    visited_files: &mut HashSet<PathBuf>,
    on_progress: &mut impl FnMut(usize, usize),
) -> Result<(), String> {
    // resolve the directory's actual location
    let canonical_dir = fs::canonicalize(path)
        .map_err(|error| format!("Cannot resolve '{}': {}", path.display(), error))?;

    // Prevent repeated visits and circular directory traversal.
    if visited_dirs.contains(&canonical_dir) {
        return Ok(());
    }

    let directory = fs::read_dir(path)
        .map_err(|error| format!("Cannot read '{}': {}", path.display(), error))?;

    visited_dirs.insert(canonical_dir);

    // count each successfully opened directory.
    report.directories_scanned += 1;

    on_progress(report.directories_scanned, report.files.len());

    let mut entries = Vec::new();

    for entry in directory {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                report.warnings.push(format!(
                    "Cannot read entry in '{}': {}",
                    path.display(),
                    error
                ));
                continue;
            }
        };

        let entry_path = entry.path();

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                report.warnings.push(format!(
                    "Cannot inspect '{}': {}",
                    entry_path.display(),
                    error
                ));
                continue;
            }
        };

        entries.push((entry_path, file_type));
    }

    // Visit ordinary entries before symbolic links.
    entries.sort_by(|a, b| {
        a.1.is_symlink()
            .cmp(&b.1.is_symlink())
            .then_with(|| a.0.cmp(&b.0))
    });

    for (entry_path, file_type) in entries {
        if file_type.is_symlink() && !follow_links {
            continue;
        }

        let resolved_type = if file_type.is_symlink() {
            match fs::metadata(&entry_path) {
                Ok(metadata) => metadata.file_type(),
                Err(error) => {
                    report.warnings.push(format!(
                        "Cannot follow '{}': {}",
                        entry_path.display(),
                        error
                    ));
                    continue;
                }
            }
        } else {
            file_type
        };

        if resolved_type.is_dir() {
            let can_descend = match max_depth {
                Some(limit) => current_depth < limit,
                None => true,
            };

            if can_descend {
                if let Err(error) = scan_recursive(
                    &entry_path,
                    report,
                    current_depth + 1,
                    max_depth,
                    follow_links,
                    visited_dirs,
                    visited_files,
                    on_progress,
                ) {
                    report.warnings.push(error);
                }
            }
        } else if resolved_type.is_file() {
            if follow_links {
                let canonical_file = match fs::canonicalize(&entry_path) {
                    Ok(path) => path,
                    Err(error) => {
                        report.warnings.push(format!(
                            "Cannot resolve '{}': {}",
                            entry_path.display(),
                            error
                        ));
                        continue;
                    }
                };

                if !visited_files.insert(canonical_file) {
                    continue;
                }
            }

            report.files.push(entry_path);

            on_progress(report.directories_scanned, report.files.len());
        }
    }

    Ok(())
}

/// get the size of a regular file in bytes
pub fn get_file_size(path: &Path) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|error| {
        format!(
            "Cannot retrieve metadata for '{}': {}",
            path.display(),
            error
        )
    })?;

    if !metadata.is_file() {
        return Err(format!("Not a regular file: {}", path.display()));
    }

    Ok(metadata.len())
}

#[cfg(test)]
mod tests {
    use super::scan_directory;
    use std::path::PathBuf;

    #[test]
    fn rejects_missing_root_directory() {
        let path = std::env::temp_dir()
            .join(format!("diskscope-missing-root-{}", std::process::id()))
            .join("nonexistent");

        let result = scan_directory(&path, None, false, |_, _| {});
        assert!(result.is_err());
    }

    #[test]
    fn scans_existing_directory_without_failing() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let result = scan_directory(&path, None, false, |_, _| {});
        assert!(result.is_ok());
    }

    #[test]
    fn depth_zero_counts_only_root_directory() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let report = scan_directory(&path, Some(0), false, |_, _| {})
            .expect("Root directory should be readable");
        assert_eq!(report.directories_scanned, 1);
    }

    #[test]
    fn reports_scan_progress() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let mut updates = Vec::new();

        let report = scan_directory(&path, Some(0), false, |directories, files| {
            updates.push((directories, files));
        })
        .expect("Scanning should succeed");

        assert!(!updates.is_empty());

        assert_eq!(
            updates.last().copied(),
            Some((report.directories_scanned, report.files.len()))
        );
    }
}
