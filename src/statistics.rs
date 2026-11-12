use std::collections::BTreeMap;
use std::path::PathBuf;

/// stores the number of files and total size for one extension
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ExtensionStat {
    pub file_count: usize,
    pub total_bytes: u64,
}

/// groups analysed files by extension & sorts by total size
pub fn extension_statistics(files: &[(PathBuf, u64)]) -> Vec<(String, ExtensionStat)> {
    let mut statistics: BTreeMap<String, ExtensionStat> = BTreeMap::new();

    for (path, size) in files {
        let extension = match path.extension() {
            Some(ext) if !ext.is_empty() => match ext.to_str() {
                Some(text) => format!(".{}", text.to_ascii_lowercase()),
                None => "[non-UTF-8 extension]".to_string(),
            },
            _ => "[no extension]".to_string(),
        };

        let stat = statistics.entry(extension).or_default();

        stat.file_count += 1;
        stat.total_bytes += size;
    }

    let mut results: Vec<_> = statistics.into_iter().collect();

    // sort by total storage usage largest first
    // use extension name to break ties consistently
    results.sort_by(|a, b| {
        b.1.total_bytes
            .cmp(&a.1.total_bytes)
            .then_with(|| a.0.cmp(&b.0))
    });

    results
}

#[cfg(test)]
mod tests {
    use super::extension_statistics;
    use std::path::PathBuf;

    #[test]
    fn groups_same_extensions() {
        let files = vec![
            (PathBuf::from("report.pdf"), 1024),
            (PathBuf::from("invoice.PDF"), 2048),
            (PathBuf::from("notes.txt"), 512),
        ];

        let results = extension_statistics(&files);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, ".pdf");
        assert_eq!(results[0].1.file_count, 2);
        assert_eq!(results[0].1.total_bytes, 3072);
    }

    #[test]
    fn handles_files_without_extensions() {
        let files = vec![
            (PathBuf::from("README"), 500),
            (PathBuf::from("LICENSE"), 300),
        ];

        let results = extension_statistics(&files);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "[no extension]");
        assert_eq!(results[0].1.file_count, 2);
        assert_eq!(results[0].1.total_bytes, 800);
    }

    #[test]
    fn sorts_extensions_by_total_size() {
        let files = vec![
            (PathBuf::from("document.pdf"), 100),
            (PathBuf::from("video.mp4"), 5000),
            (PathBuf::from("photo.png"), 2000),
        ];

        let results = extension_statistics(&files);

        assert_eq!(results[0].0, ".mp4");
        assert_eq!(results[1].0, ".png");
        assert_eq!(results[2].0, ".pdf");
    }

    #[test]
    fn handles_empty_file_collection() {
        let files = Vec::new();

        let results = extension_statistics(&files);

        assert!(results.is_empty());
    }
}
