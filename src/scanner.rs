use std::fs;
use std::io;
use std::path::Path;

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
        let entry = entry.map_err(|error| {
            format!("Cannot read entry in '{}': {}", path.display(), error)
        })?;

        entries.push(entry);
    }

    // Sort entries alphabetically for consistent output.
    entries.sort_by_key(|entry| {
        entry.file_name().to_string_lossy().to_lowercase()
    });

    Ok(entries)
}