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
pub fn scan_directory(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    scan_recursive(path, &mut files)?;

    // ensure consistent output regardless of filesystem order
    files.sort();

    Ok(files)
}

/// visits a directory & its subdirectories
fn scan_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(path)
        .map_err(|error| format!("Cannot read '{}': {}", path.display(), error))?;

    for entry in entries {
        let entry = entry
            .map_err(|error| format!("Cannot read entry in '{}': {}", path.display(), error))?;

        let entry_path = entry.path();

        let file_type = entry
            .file_type()
            .map_err(|error| format!("Cannot inspect '{}': {}", entry_path.display(), error))?;

        // avoid following symbolic links
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            // explore the subdirectory
            scan_recursive(&entry_path, files)?;
        } else if file_type.is_file() {
            // store the discovered file
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
