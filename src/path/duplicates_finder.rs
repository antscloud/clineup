use crate::errors::ClineupError;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

/// Calculates the SHA256 hash of a given file by using a 1024 bytes buffer.
///
/// # Arguments
///
/// * `open_file` - The file to calculate the hash of.
///
/// # Returns
///
/// The SHA256 hash of the file as a hexadecimal string, or an error if the hash calculation fails.
fn get_hash_of_file(mut open_file: File) -> Result<String, ClineupError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1024];
    loop {
        match open_file.read(&mut buffer) {
            Ok(0) => break, // End of file
            Ok(bytes_read) => {
                hasher.update(&buffer[..bytes_read]);
            }
            Err(_) => {
                return Err(ClineupError::HashError(
                    "Something went wrong reading buffer of file".to_string(),
                ))
            }
        }
    }
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub struct DuplicatesFinder {
    _duplicates: HashMap<u64, Vec<String>>,
}

impl DuplicatesFinder {
    pub fn new() -> Self {
        DuplicatesFinder {
            _duplicates: HashMap::new(),
        }
    }
}

/// Checks if the given path is a duplicate file by detecting its
/// precence in the hashmap.Otherwise store the hash of the file in the hashmap
///
/// # Arguments
///
/// * `path` - A `PathBuf` representing the path to the file.
///
/// # Returns
///
/// Returns `Ok(true)` if the file is a duplicate, `Ok(false)` otherwise.
/// Returns an `Err` if there was an error while checking for duplicates.
impl DuplicatesFinder {
    pub fn is_duplicate(&mut self, path: &PathBuf) -> Result<bool, ClineupError> {
        let metadata = std::fs::metadata(path)?;

        if metadata.is_dir() {
            return Ok(false);
        }

        if metadata.len() == 0 {
            return Ok(false);
        }

        let open_file = File::open(path)?;
        let hash_of_file = get_hash_of_file(open_file)?;

        if !self._duplicates.contains_key(&metadata.len()) {
            self._duplicates.insert(metadata.len(), vec![hash_of_file]);
            return Ok(false);
        }

        if let Some(duplicates) = self._duplicates.get_mut(&metadata.len()) {
            if !duplicates.contains(&hash_of_file) {
                duplicates.push(hash_of_file);
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        Ok(true)
    }
}
