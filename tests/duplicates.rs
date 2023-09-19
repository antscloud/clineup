#[cfg(test)]
mod tests {
    use std::{fs::File, path::PathBuf};

    use clineup::path::duplicates_finder::{get_hash_of_file, DuplicatesFinder};

    #[test]
    fn test_get_hash_of_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/London.png");
        let file = File::open(path).unwrap();
        let hash = get_hash_of_file(&file).unwrap();
        // Add assertions for the expected hash value
        assert_eq!(
            hash,
            "bb8f2afe981cbd9ebb3c38ab7ad24042385fe6701f703be14771c4cfe77d4679"
        );
    }

    #[test]
    fn test_duplicates_finder() {
        let mut duplicates_finder = DuplicatesFinder::new();

        // Test case: Non-duplicate file
        let path1 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/Paris-20230304.jpg");
        let is_duplicate1 = duplicates_finder.is_duplicate(&path1).unwrap();
        assert!(!is_duplicate1);

        // Test case: Duplicate file with same size and hash
        let path2 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/London.png");
        let is_duplicate2 = duplicates_finder.is_duplicate(&path2).unwrap();
        assert!(!is_duplicate2);

        // Test case: Duplicate file with same size but different hash
        let path3 = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/Paris-20230304-duplicated.jpg");
        let is_duplicate3 = duplicates_finder.is_duplicate(&path3).unwrap();
        assert!(is_duplicate3);
    }
}
