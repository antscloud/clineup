use std::{collections::HashMap, process::exit};

use log::error;

use crate::cli::Config;
use crate::gps::base::GpsResolutionProvider;
use crate::gps::gpsenum::GpsResolutionProviderImpl;
use crate::gps::nominatim::Nominatim;
use crate::organizer::CopyStrategy;
use crate::organizer::MoveStrategy;
use crate::organizer::OrganizationMode;
use crate::organizer::OrganizationStrategy;
use crate::organizer::SymlinksStrategy;
use crate::{errors::ClineupError, placeholders::Placeholder};
use path_clean::{clean, PathClean};
use std::path::Path;
use std::path::PathBuf;

/// Checks if there is an EXIF placeholder in the given placeholders.
/// Returns `true` if there is at least one EXIF placeholder, `false` otherwise.
pub fn is_there_a_exif_placeholder(
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) -> bool {
    let mut is_there_a_date_placeholder = false;

    for _placeholders in placeholders.values() {
        for _placeholder in _placeholders.values() {
            if _placeholder.is_exif_related() {
                is_there_a_date_placeholder = true;
                break;
            }
        }
    }

    is_there_a_date_placeholder
}

/// Checks if there is an metadata placeholder in the given placeholders.
/// Returns `true` if there is at least one metadata placeholder, `false` otherwise.
pub fn is_there_a_metadata_placeholder(
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) -> bool {
    let mut is_there_a_metadata_placeholder = false;
    for _placeholders in placeholders.values() {
        for _placeholder in _placeholders.values() {
            if _placeholder.is_os_related() {
                is_there_a_metadata_placeholder = true;
                break;
            }
        }
    }
    is_there_a_metadata_placeholder
}
/// Checks if there is an location related placeholder in the given placeholders.
/// Returns `true` if there is at least one location related placeholder, `false` otherwise.
pub fn is_there_a_location_placeholder(
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) -> bool {
    let mut is_there_a_location_placeholder = false;
    for _placeholders in placeholders.values() {
        for _placeholder in _placeholders.values() {
            if _placeholder.is_location_related() {
                is_there_a_location_placeholder = true;
                break;
            }
        }
    }
    is_there_a_location_placeholder
}

pub fn print_error<T>(e: ClineupError) -> T {
    error!("{e}");
    exit(1)
}

/// Constructs a full format path based on the provided folder and filename formats.
///
/// # Arguments
///
/// * `folder_format` - An optional string representing the folder format.
/// * `filename_format` - An optional string representing the filename format.
///
/// # Returns
///
/// An optional `String` representing the full format path, or `None` if both `folder_format` and `filename_format` are `None`.
pub fn get_full_format_path(
    folder_format: Option<&String>,
    filename_format: Option<&String>,
) -> Option<String> {
    let mut full_format: Option<PathBuf> = None;
    if let Some(_folder_format) = folder_format {
        let mut _full_format = Path::new(&_folder_format).to_path_buf().clean();
        if let Some(_filename_format) = filename_format {
            _full_format = _full_format.join(_filename_format);
            full_format = Some(_full_format);
        } else {
            _full_format = _full_format.join("{%original_filename}");
            full_format = Some(_full_format);
        }
    } else if let Some(_filename_format) = filename_format {
        full_format = Some(Path::new(&_filename_format).to_path_buf().clean());
    }

    // Convert the final PathBuf into an Option<String>
    full_format.map(|path| path.to_string_lossy().to_string())
}

/// Retrieves the reverse geocoding provider based on the given configuration.
///
/// # Arguments
///
/// * `config` - The configuration object.
///
/// # Returns
///
/// An `Option<Box<dyn GpsResolutionProvider>>` representing the reverse geocoding provider.
pub fn get_reverse_geocoding(config: &Config) -> Option<Box<dyn GpsResolutionProvider>> {
    match &config.reverse_geocoding {
        Some(provider) => match provider {
            GpsResolutionProviderImpl::Nominatim => {
                // Check if Nominatim email is provided
                if config.nominatim_email.is_none() {
                    println!("Nominatim email is required when using Nominatim as a reverse geocoding provider.");
                    exit(1)
                }
                // Create a new Nominatim instance with the provided email
                Some(Box::new(Nominatim::new(
                    config.nominatim_email.clone().unwrap(),
                )))
            }
        },
        None => None,
    }
}

pub fn get_organization_strategy(
    strategy: Option<&OrganizationMode>,
) -> Option<Box<dyn OrganizationStrategy>> {
    match strategy {
        Some(org_mode) => match org_mode {
            OrganizationMode::Copy => Some(Box::new(CopyStrategy::new())),
            OrganizationMode::Symlinks => Some(Box::new(SymlinksStrategy::new())),
            OrganizationMode::Move => Some(Box::new(MoveStrategy::new())),
        },
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_full_format_path() {
        // Test case 1: Both folder_format and filename_format are None
        assert_eq!(get_full_format_path(None, None), None);

        // Test case 2: Only folder_format is provided
        assert_eq!(
            get_full_format_path(Some(&String::from("path/to/folder")), None),
            Some(String::from("path/to/folder/{%original_filename}"))
        );

        // Test case 3: Only filename_format is provided
        assert_eq!(
            get_full_format_path(None, Some(&String::from("file.txt"))),
            Some(String::from("file.txt"))
        );

        // Test case 4: Both folder_format and filename_format are provided
        assert_eq!(
            get_full_format_path(
                Some(&String::from("path/to/folder")),
                Some(&String::from("file.txt"))
            ),
            Some(String::from("path/to/folder/file.txt"))
        );

        // Test case 5: Only filename_format is provided, with placeholder
        assert_eq!(
            get_full_format_path(None, Some(&String::from("%original_filename"))),
            Some(String::from("%original_filename"))
        );

        // Test case 6: Both folder_format and filename_format are provided, with placeholder
        assert_eq!(
            get_full_format_path(
                Some(&String::from("path/to/folder")),
                Some(&String::from("%original_filename"))
            ),
            Some(String::from("path/to/folder/%original_filename"))
        );
    }

    #[test]
    fn test_get_organization_strategy_copy() {
        let strategy = OrganizationMode::Copy;
        let result = get_organization_strategy(Some(&strategy));
        assert!(result.is_some());
    }

    #[test]
    fn test_get_organization_strategy_symlinks() {
        let strategy = OrganizationMode::Symlinks;
        let result = get_organization_strategy(Some(&strategy));
        assert!(result.is_some());
    }

    #[test]
    fn test_get_organization_strategy_move() {
        let strategy = OrganizationMode::Move;
        let result = get_organization_strategy(Some(&strategy));
        assert!(result.is_some());
    }

    #[test]
    fn test_get_organization_strategy_none() {
        let result = get_organization_strategy(None);
        assert!(result.is_none());
    }
}
