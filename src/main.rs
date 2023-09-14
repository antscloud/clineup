use clineup::cli::check_cli_config_from_placeholders;
use clineup::cli::get_cli_config;
use clineup::cli::parse_cli;
use clineup::cli::set_log_level;
use clineup::cli::Config;
use clineup::gps::base::GpsResolutionProvider;
use clineup::gps::gpsenum::GpsResolutionProviderImpl;
use clineup::gps::nominatim::Nominatim;
use clineup::organizer::CopyStrategy;
use clineup::organizer::MoveStrategy;
use clineup::organizer::OrganizationMode;
use clineup::organizer::OrganizationStrategy;
use clineup::organizer::SymlinksStrategy;
use clineup::path::duplicates_finder::DuplicatesFinder;
use clineup::path::formatter::PathFormatter;
use clineup::path::iterator::FileIterator;
use clineup::path::parser::map_placeholders_to_enums;
use clineup::path::parser::parse_placeholders;

use log::debug;
use log::error;
use log::info;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

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
fn get_full_format_path(
    folder_format: Option<&String>,
    filename_format: Option<&String>,
) -> Option<String> {
    let mut full_format: Option<PathBuf> = None;

    if let Some(folder_format) = folder_format {
        full_format = Some(Path::new(&folder_format).to_path_buf());
    }

    if let Some(filename_format) = filename_format {
        let _format = Path::new(&filename_format).to_path_buf();
        if let Some(mut outer_full_format) = full_format.take() {
            outer_full_format.push(_format);
            full_format = Some(outer_full_format);
        } else {
            full_format = Some(_format);
        }
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
                    error!("Nominatim email is required when using Nominatim as a reverse geocoding provider.");
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

/// Returns an organization strategy based on the given config.
///
/// # Arguments
///
/// * `config` - The configuration object.
///
/// # Returns
///
/// An `Option` containing a boxed organization strategy, or `None` if no strategy is specified in the config.
pub fn get_organization_strategy(config: &Config) -> Option<Box<dyn OrganizationStrategy>> {
    match &config.strategy {
        Some(strategy) => match strategy {
            OrganizationMode::Copy => Some(Box::new(CopyStrategy::new())),
            OrganizationMode::Symlinks => Some(Box::new(SymlinksStrategy::new())),
            OrganizationMode::Move => Some(Box::new(MoveStrategy::new())),
        },
        None => None,
    }
}

fn main() {
    let matches = parse_cli();

    set_log_level(matches.occurrences_of("verbose"));

    let config = get_cli_config(matches);

    let strategy = get_organization_strategy(&config);

    if strategy.is_none() {
        error!("Strategy is not set");
        exit(1)
    } else {
        debug!("Get strategy {:?}", config.strategy);
    }

    let full_path = get_full_format_path(
        config.folder_format.as_ref(),
        config.filename_format.as_ref(),
    );

    if full_path.is_none() {
        error!("You should provide at least one of the folder or filename format.");
        exit(1);
    } else {
        debug!("Full path {:?}", full_path);
    }

    debug!("Parsing placeholders");
    let _placeholders = parse_placeholders(&full_path.as_ref().unwrap());
    let placeholders = map_placeholders_to_enums(&_placeholders);
    check_cli_config_from_placeholders(&config, &placeholders);

    let destination = Path::new(&config.destination);

    let reverse_geocoding = get_reverse_geocoding(&config);
    debug!("Get reverse geocoding strategy");

    // It is mutable to be able to store the positions and location when optmizing gps positions
    let mut path_formatter = PathFormatter::new(
        full_path.as_ref().unwrap(),
        &placeholders,
        reverse_geocoding,
        config.gps_optimization,
    );
    let files = FileIterator::new(&config);

    let mut duplicates_finder = if config.drop_duplicates {
        Some(DuplicatesFinder::new())
    } else {
        None
    };

    if config.dry_run {
        info!("Configuration \n{:?}", config)
    }
    let mut dry_run_count = 0;

    for entry in files {
        if config.drop_duplicates {
            let is_duplicate = duplicates_finder.as_mut().unwrap().is_duplicate(&entry);

            match is_duplicate {
                Ok(is_duplicate) => {
                    if is_duplicate {
                        info!("Find duplicate {:?}", entry.display());
                        continue;
                    }
                }
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            }
        }

        let formatted_path = path_formatter.get_formatted_path(&entry);

        let good_formatted_path = match formatted_path {
            Ok(path) => destination.join(path),
            Err(err) => {
                error!("{}", err);
                continue;
            }
        };

        debug!("Get formatted path {:?}", good_formatted_path);

        if config.dry_run {
            if dry_run_count >= config.dry_run_number_of_files {
                exit(0);
            }
            info!("{:?} -> {}", entry, good_formatted_path.display());
            dry_run_count += 1;
            continue;
        }
        strategy
            .as_ref()
            .unwrap()
            .organize(&entry, &good_formatted_path);
    }
}
