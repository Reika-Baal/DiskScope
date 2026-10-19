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

/// recursively scans a directory & returns all regular file paths
/// optional maximum depth limit how deeply subdirectories are explored
pub fn scan_directory(
    path: &Path,
    max_depth: Option<usize>,
    follow_links: bool,
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    // track directories and files to prevent repeated visits when following symbolic links
    let mut visited_dirs: HashSet<PathBuf> = HashSet::new();
    let mut visited_files: HashSet<PathBuf> = HashSet::new();

    scan_recursive(
        path,
        &mut files,
        0,
        max_depth,
        follow_links,
        &mut visited_dirs,
        &mut visited_files,
    )?;

    files.sort();

    Ok(files)
}

/// visits a directory & its subdirectories
fn scan_recursive(
    path: &Path,
    files: &mut Vec<PathBuf>,
    current_depth: usize,
    max_depth: Option<usize>,
    follow_links: bool,
    visited_dirs: &mut HashSet<PathBuf>,
    visited_files: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    // Resolve the directory's actual location.
    let canonical_dir = fs::canonicalize(path)
        .map_err(|error| format!("Cannot resolve '{}': {}", path.display(), error))?;

    // Prevent repeated visits and circular directory traversal.
    if !visited_dirs.insert(canonical_dir) {
        return Ok(());
    }

    let directory = fs::read_dir(path)
        .map_err(|error| format!("Cannot read '{}': {}", path.display(), error))?;

    let mut entries = Vec::new();

    for entry in directory {
        let entry = entry
            .map_err(|error| format!("Cannot read entry in '{}': {}", path.display(), error))?;

        let file_type = entry
            .file_type()
            .map_err(|error| format!("Cannot inspect '{}': {}", entry.path().display(), error))?;

        entries.push((entry.path(), file_type));
    }

    // Process ordinary entries before symbolic links.
    entries.sort_by(|a, b| {
        a.1.is_symlink()
            .cmp(&b.1.is_symlink())
            .then_with(|| a.0.cmp(&b.0))
    });

    for (entry_path, file_type) in entries {
        // skip symbolic links unless user explicitly enables
        if file_type.is_symlink() && !follow_links {
            continue;
        }

        // resolve the target type when following a symbolic link
        let resolved_type = if file_type.is_symlink() {
            fs::metadata(&entry_path)
                .map_err(|error| format!("Cannot follow '{}': {}", entry_path.display(), error))?
                .file_type()
        } else {
            file_type
        };

        if resolved_type.is_dir() {
            let can_descend = match max_depth {
                Some(limit) => current_depth < limit,
                None => true,
            };

            if can_descend {
                scan_recursive(
                    &entry_path,
                    files,
                    current_depth + 1,
                    max_depth,
                    follow_links,
                    visited_dirs,
                    visited_files,
                )?;
            }
        } else if resolved_type.is_file() {
            // avoid counting symbolic link more than once
            if follow_links {
                let canonical_file = fs::canonicalize(&entry_path).map_err(|error| {
                    format!("Cannot resolve '{}': {}", entry_path.display(), error)
                })?;

                if !visited_files.insert(canonical_file) {
                    continue;
                }
            }

            files.push(entry_path);
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
