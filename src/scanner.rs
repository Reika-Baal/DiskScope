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
