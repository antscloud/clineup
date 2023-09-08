use std::collections::HashMap;
use std::process::exit;

use crate::errors::ClineupError;
use crate::gps::gpsenum::GpsResolutionProviderImpl;
use crate::organizer::OrganizationMode;
use crate::placeholders::Placeholder;
use crate::utils::is_there_a_location_placeholder;
use crate::utils::print_error;
use clap::{App, Arg};
use env_logger;
use log::{error, LevelFilter};
use regex::Regex;

// Configuration struct for the photo organizer
#[derive(Debug, Clone)]
pub struct Config {
    pub source: String,
    pub destination: String,
    pub recursive: bool,
    pub extensions: Option<Vec<String>>,
    pub exclude_extensions: Option<Vec<String>>,
    pub include_regex: Option<Regex>,
    pub exclude_regex: Option<Regex>,
    pub size_greater: Option<u64>,
    pub size_lower: Option<u64>,
    pub dry_run: bool,
    pub dry_run_number_of_files: u64,
    pub log_file: Option<String>,
    pub verbosity: u64,
    pub drop_duplicates: bool,
    pub strategy: Option<OrganizationMode>,
    pub reverse_geocoding: Option<GpsResolutionProviderImpl>,
    pub nominatim_email: Option<String>,
    pub folder_format: Option<String>,
    pub filename_format: Option<String>,
}

// Define the command-line parameters using the 'clap' crate
fn define_cli_parameters() -> App<'static, 'static> {
    App::new("Clineup").about("Utility tool for organizing media")
        .arg(
            Arg::with_name("source")
                .long("source")
                .value_name("SOURCE")
                .help("Specifies the source directory or file to be organized")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("destination")
                .long("destination")
                .value_name("DESTINATION")
                .help(
                    "Specifies the destination directory where the organized photos will be stored",
                )
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("recursive")
                .long("recursive")
                .help("Performs the organization process recursively on subdirectories"),
        )
        .arg(
            Arg::with_name("extension")
                .long("extension")
                .value_name("EXTENSION")
                .help("Filters photos based on file extensions")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("exclude-extension")
                .long("exclude-extension")
                .value_name("EXTENSION")
                .help("Excludes photos with the specified file extensions")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("include-regex")
                .long("include-regex")
                .value_name("INCLUDE-REGEX")
                .help("The include regex is used to filter files")
                .long_help("
                The regex is matched against the full path of the file, including the parent folders.\n
                For example, to include all files containing 'IMG', use the regex '.*IMG.*")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("exclude-regex")
                .long("exclude-regex")
                .value_name("EXCLUDE-REGEX")
                .help("The exclude regex is used to filter files")
                .long_help("
                The regex is matched against the full path of the file, including the parent folders.\n
                For example, to exclude all files containing 'IMG', use the regex '.*IMG.*")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("size-greater")
                .long("size-greater")
                .value_name("SIZE")
                .help("Filters photos greater than the specified size. Use 'KB', 'MB', 'GB', 'TB' or 'PB'")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("size-lower")
                .long("size-lower")
                .value_name("SIZE")
                .help("Filters photos lower than the specified size. Use 'KB', 'MB', 'GB', 'TB' or 'PB'")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dry-run")
                .long("dry-run")
                .help("Performs a dry run without actually moving or renaming any files"),
        )
        .arg(
            Arg::with_name("dry-run-number-of-files")
                .long("dry-run-number-of-files")
                .help("Specifies the number of files to be processed by the dry run"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true) // Allow multiple occurrences of -v (i.e., -vv, -vvv)
                .help("Sets the log level to increase verbosity"),
        )
        .arg(
            Arg::with_name("folder-format")
                .long("folder-format")
                .help("Specifies the folder format to create"),
        )
        .arg(
            Arg::with_name("filename-format")
                .long("filename-format")
                .takes_value(true)
                .help("Specifies the filename format to create"),
        )
        .arg(
            Arg::with_name("gps-optimization")
                .long("gps-optimization")
                .help("Round the lat ang long to 1 decimal places. It becomes less accurate (about 1 kilometer) but can save a lot of API calls.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("strategy")
                .long("strategy")
                .help("Specifies the organization strategy")
                .possible_values(&["copy", "symlink", "move"])
                .default_value("copy")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("drop-duplicates")
                .long("drop-duplicates")
                .help("Drop duplicates depending on the strategy")
                .long_help("Drop duplicates depending on the strategy \n
                - Copy : Do not copy the duplicates \n
                - Symlink : Do not symlink the duplicates \n
                - Move : Do not move the duplicates
                ")
        )
        .arg(
            Arg::with_name("reverse-geocoding")
                .long("reverse-geocoding")
                .help("Reverse geocoding provider to use")
                .possible_values(&["nominatim"])
                .takes_value(true),
        )
        .arg(
            Arg::with_name("nominatim-email")
                .long("nominatim-email")
                .help("Email to use for nominatim API. This is mandatory following the nominatim usage policy")
                .takes_value(true),
        )
}

fn convert_size_to_bytes(size: &str) -> Result<u64, ClineupError> {
    let re = Regex::new(r"(?P<number>[0-9]+)(?P<unit>[KMGTP]?)[Bo]?")?;

    if let Some(capture) = re.captures(size) {
        if let Some(number_str) = capture.name("number") {
            if number_str.len() == 0 {
                return Err(ClineupError::InvalidSizeFormat(size.to_string()));
            }

            let number: u64 = number_str
                .as_str()
                .parse()
                .map_err(|_| ClineupError::InvalidNumberFormat(number_str.as_str().to_string()))?;

            if let Some(unit) = capture.name("unit") {
                if unit.len() == 0 {
                    return Err(ClineupError::InvalidSizeFormat(size.to_string()));
                }
                let unit_str = unit.as_str();
                let multiplier = match unit_str {
                    _ if unit_str.contains('K') => 1024,
                    _ if unit_str.contains('M') => 1024 * 1024,
                    _ if unit_str.contains('G') => 1024 * 1024 * 1024,
                    _ if unit_str.contains('T') => 1024 * 1024 * 1024 * 1024,
                    _ if unit_str.contains('P') => 1024 * 1024 * 1024 * 1024 * 1024,
                    _ => 1,
                };
                return Ok(number * multiplier);
            } else {
                return Err(ClineupError::InvalidSizeFormat(size.to_string()));
            }
        }
    }
    Err(ClineupError::InvalidSizeFormat(size.to_string()))
}

pub fn set_log_level(verbosity: u64) {
    let level = match verbosity {
        0 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };

    env_logger::builder().filter_level(level).init();
}

fn get_geocoding_enum(
    _enum: Option<&str>,
) -> Result<Option<GpsResolutionProviderImpl>, ClineupError> {
    if let Some(_good_enum) = _enum {
        match _good_enum {
            "nominatim" => Ok(Some(GpsResolutionProviderImpl::Nominatim)),
            _ => Err(ClineupError::InvalidGeocodingProvider(
                _good_enum.to_string(),
            )),
        }
    } else {
        Ok(None)
    }
}
fn get_strategy_enum(_enum: Option<&str>) -> Result<Option<OrganizationMode>, ClineupError> {
    if let Some(_good_enum) = _enum {
        match _good_enum {
            "copy" => Ok(Some(OrganizationMode::Copy)),
            "symlink" => Ok(Some(OrganizationMode::Symlinks)),
            "move" => Ok(Some(OrganizationMode::Move)),
            _ => Err(ClineupError::InvalidOrganization(_good_enum.to_string())),
        }
    } else {
        Ok(None)
    }
}
fn get_size_greater(size_greater: Option<&str>) -> Result<Option<u64>, ClineupError> {
    if let Some(size_gt) = size_greater {
        convert_size_to_bytes(size_gt).map(|r| Some(r))
    } else {
        Ok(None)
    }
}
fn get_size_lower(size_lower: Option<&str>) -> Result<Option<u64>, ClineupError> {
    if let Some(size_lt) = size_lower {
        convert_size_to_bytes(size_lt).map(|r| Some(r))
    } else {
        Ok(None)
    }
}

fn convert_to_regex(regex: Option<&str>) -> Result<Option<Regex>, ClineupError> {
    if let Some(regex_str) = regex {
        Ok(Some(Regex::new(regex_str)?))
    } else {
        Ok(None)
    }
}

pub fn parse_cli() -> clap::ArgMatches<'static> {
    define_cli_parameters().get_matches()
}

pub fn get_cli_config(matches: clap::ArgMatches) -> Config {
    // Unwrap is used here to quit as soon as possible whern parsing
    let size_greater =
        get_size_greater(matches.value_of("size-greater")).unwrap_or_else(print_error);
    let size_lower = get_size_lower(matches.value_of("size-lower")).unwrap_or_else(print_error);
    let reverse_geocoding =
        get_geocoding_enum(matches.value_of("reverse-geocoding")).unwrap_or_else(print_error);
    let strategy = get_strategy_enum(matches.value_of("strategy")).unwrap_or_else(print_error);

    let dry_number_of_files_str = matches.value_of("dry-run-number-of-files").unwrap_or("10");
    let dry_number_of_files = dry_number_of_files_str.parse::<u64>().unwrap_or(10);

    let include_regex =
        convert_to_regex(matches.value_of("include-regex")).unwrap_or_else(print_error);
    let exclude_regex =
        convert_to_regex(matches.value_of("exclude-regex")).unwrap_or_else(print_error);

    Config {
        source: matches.value_of("source").unwrap().to_string(),
        destination: matches.value_of("destination").unwrap().to_string(),
        recursive: matches.is_present("recursive"),
        extensions: matches
            .values_of("extension")
            .map(|values| values.map(|e| e.to_ascii_lowercase()).collect()),
        exclude_extensions: matches
            .values_of("exclude-extension")
            .map(|values| values.map(|e| e.to_ascii_lowercase()).collect()),
        include_regex: include_regex,
        exclude_regex: exclude_regex,
        size_greater: size_greater,
        size_lower: size_lower,
        dry_run: matches.is_present("dry-run"),
        dry_run_number_of_files: dry_number_of_files,
        log_file: matches.value_of("log").map(|log| log.to_string()),
        verbosity: matches.occurrences_of("verbose"),
        strategy,
        drop_duplicates: matches.is_present("drop-duplicates"),
        reverse_geocoding,
        nominatim_email: matches
            .value_of("nominatim-email")
            .map(|email| email.to_string()),
        folder_format: matches
            .value_of("folder-format")
            .map(|folder_format| folder_format.to_string()),
        filename_format: matches
            .value_of("filename-format")
            .map(|filename_format| filename_format.to_string()),
    }
}

pub fn check_cli_config_from_placeholders(
    config: &Config,
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) {
    if is_there_a_location_placeholder(placeholders) {
        if config.reverse_geocoding.is_none() {
            error!("Location tag found but reverse geocoding provider is not set");
            exit(1)
        }
    }
}
