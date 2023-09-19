#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clineup::path::iterator::is_allowed_extension;
    use clineup::path::iterator::is_allowed_size;

    #[test]
    fn test_is_allowed_extension_allowed() {
        let entry = PathBuf::from("file.txt");
        let extensions = Some(vec!["txt".to_string()]);
        let exclude_extensions = None;

        assert!(is_allowed_extension(
            &entry,
            &extensions,
            &exclude_extensions
        ));
    }

    #[test]
    fn test_is_allowed_extension_not_allowed() {
        let entry = PathBuf::from("file.jpg");
        let extensions = Some(vec!["txt".to_string()]);
        let exclude_extensions = None;

        assert!(!is_allowed_extension(
            &entry,
            &extensions,
            &exclude_extensions
        ));
    }
    #[test]
    fn test_file_size_greater_than_lower() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/5384_bytes.png");
        let size_lower = Some(100);
        let size_greater = None;

        let result = is_allowed_size(&path, &size_lower, &size_greater);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_file_size_lower_than_greater() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/5384_bytes.png");
        let size_lower = None;
        let size_greater = Some(10000);

        let result = is_allowed_size(&path, &size_lower, &size_greater);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_file_size_within_bounds() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/5384_bytes.png");
        let size_lower = Some(10000);
        let size_greater = Some(100);

        let result = is_allowed_size(&path, &size_lower, &size_greater);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
