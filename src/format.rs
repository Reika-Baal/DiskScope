/// convert file size in bytes into a readable string
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

    let mut size = bytes as f64;
    let mut unit_index = 0;

    // move to next unit whenever the size reaches 1024
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    // show whole numbers for bytes
    if unit_index == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::format_size;

    #[test]
    fn formats_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    fn formats_kibibytes() {
        assert_eq!(format_size(1024), "1.00 KiB");
        assert_eq!(format_size(1536), "1.50 KiB");
    }

    #[test]
    fn formats_larger_units() {
        assert_eq!(format_size(1_048_576), "1.00 MiB");
        assert_eq!(format_size(1_073_741_824), "1.00 GiB");
    }
}
