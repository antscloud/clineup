use crate::cli::Config;
use crate::errors::ClineupError;
use glob::glob;
use log::debug;
use log::info;
use log::warn;

use std::fs;
use std::path::PathBuf;

/// Check if the file extension of the given entry is allowed based on the provided extensions and excluded extensions.
///
/// # Arguments
///
/// * `entry` - The path to the file.
/// * `extensions` - A list of allowed extensions. If `None`, all extensions are allowed.
/// * `exclude_extensions` - A list of excluded extensions. If `None`, no extensions are excluded.
///
/// # Returns
///
/// `true` if the extension is allowed, `false` otherwise.
pub fn is_allowed_extension(
    entry: &PathBuf,
    extensions: &Option<Vec<String>>,
    exclude_extensions: &Option<Vec<String>>,
) -> bool {
    let extension = match entry.extension() {
        Some(ext) => ext.to_string_lossy().to_ascii_lowercase().to_string(),
        None => return false,
    };

    // Check if the extension is in the allowed list.
    if let Some(exts) = extensions {
        if !exts.contains(&extension) {
            debug!(
                "File extension \"{}\" of {:?} is not in the allowed list",
                extension, entry
            );
            return false;
        }
    }

    // Check if the extension is in the excluded list.
    if let Some(exclude_exts) = exclude_extensions {
        if exclude_exts.contains(&extension) {
            debug!("File extension {} is in the excluded list", extension);
            return false;
        }
    }

    true
}

/// Check if the size of the file referenced by the `entry` path is within certain bounds.
///
/// # Arguments
///
/// * `entry` - A reference to a `PathBuf` representing the file path.
/// * `size_lower` - A reference to an optional `u64` representing the lower size limit.
/// * `size_greater` - A reference to an optional `u64` representing the greater size limit.
///
/// # Returns
///
/// A boolean value indicating whether the size of the file is allowed or not.
pub fn is_allowed_size(
    entry: &PathBuf,
    size_lower: &Option<u64>,
    size_greater: &Option<u64>,
) -> Result<bool, ClineupError> {
    let metadata = fs::metadata(entry)?;

    // Check if the file size is greater than the desired lower size
    if let Some(size_lt) = size_lower {
        if metadata.len() > *size_lt {
            debug!("File size is greater than {size_lt}");
            return Ok(false);
        }
    }
    // Check if the file size is lower than the desired greater size
    if let Some(size_gt) = size_greater {
        if metadata.len() < *size_gt {
            debug!("File size is lower than {size_gt}");
            return Ok(false);
        }
    }
    Ok(true)
}

/// Generates a glob pattern based on the given source path and recursion flag.
/// If recursion is enabled, append "**/*" to the source path.
/// Otherwise, append "*".
///
/// # Arguments
///
/// * `source` - The source path to generate the glob pattern for.
/// * `recursive` - A flag indicating whether the pattern should be recursive or not.
///
/// # Returns
///
/// The generated glob pattern as a `String`.
fn get_glob_pattern(source: &String, recursive: &bool) -> String {
    let mut source = source.clone();
    if *recursive {
        source.push_str("**/*");
    } else {
        source.push_str("*");
    }

    source
}

pub struct FileIterator<'a> {
    entries: glob::Paths,
    config: &'a Config,
}

impl<'a> FileIterator<'a> {
    pub fn new(config: &'a Config) -> Self {
        let source = get_glob_pattern(&config.source, &config.recursive);

        let entries = glob(&source)
            .expect(format!("Failed to iterate through source pattern {source}").as_str());

        FileIterator { entries, config }
    }
}

impl<'a> Iterator for FileIterator<'a> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.entries.next() {
            match entry {
                Err(err) => {
                    warn!("Unable to get entry: {:?}", err);
                    continue;
                }
                Ok(entry) => {
                    // Check if the entry is a file
                    if entry.is_file() {
                        if let Some(include_regex) = &self.config.include_regex {
                            if !include_regex.is_match(&entry.to_string_lossy()) {
                                continue;
                            }
                        }

                        if let Some(exclude) = &self.config.exclude_regex {
                            if exclude.is_match(&entry.to_string_lossy()) {
                                continue;
                            }
                        }

                        if !is_allowed_extension(
                            &entry,
                            &self.config.extensions,
                            &self.config.exclude_extensions,
                        ) {
                            continue;
                        }

                        let _is_allowed_size = is_allowed_size(
                            &entry,
                            &self.config.size_lower,
                            &self.config.size_greater,
                        );

                        match _is_allowed_size {
                            Ok(allowed) => {
                                if !allowed {
                                    continue;
                                }
                            }
                            Err(err) => {
                                warn!("Unable to check file size: {:?}", err);
                                continue;
                            }
                        }

                        return Some(entry);
                    }
                }
            }
        }

        None
    }
}
