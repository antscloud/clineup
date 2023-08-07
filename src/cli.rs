use std::num::ParseIntError;

use clap::{App, Arg};
use env_logger;
use log::{error, info, warn, LevelFilter};
use regex::Regex;
use serde::{Deserialize, Serialize};

// Configuration struct for the photo organizer
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    source: String,
    destination: String,
    recursive: bool,
    extension: Option<String>,
    exclude_extension: Option<String>,
    size_greater: Option<String>,
    size_lower: Option<String>,
    delete_original: bool,
    dry_run: bool,
    log_file: Option<String>,
    verbosity: u64,
    ical: bool,
    folder_format: bool,
    filename_format: bool,
}

// Define the command-line parameters using the 'clap' crate
fn define_cli_parameters() -> App<'static, 'static> {
    App::new("Photo Organizer")
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
            Arg::with_name("exclude_extension")
                .long("exclude_extension")
                .value_name("EXTENSION")
                .help("Excludes photos with the specified file extensions")
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
            Arg::with_name("log")
                .long("log")
                .value_name("LOG_FILE")
                .help("Specifies a log file to record the organization process")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true) // Allow multiple occurrences of -v (i.e., -vv, -vvv)
                .help("Sets the log level to increase verbosity"),
        )
        .arg(
            Arg::with_name("calendar")
                .long("calendar")
                .help("Specifies a calendar (.ics) to use for %event"),
        )
        .arg(
            Arg::with_name("folder-format")
                .long("folder-format")
                .help("Specifies the folder format to create"),
        )
        .arg(
            Arg::with_name("filename-format")
                .long("filename-format")
                .help("Specifies the filename format to create"),
        )
        .arg(
            Arg::with_name("config")
                .long("config-file")
                .help("Specifies the path of the configuration file"),
        )
}

#[derive(Debug)]
enum SizeParseError {
    ParseIntError(ParseIntError),
    InvalidSizeFormat,
}

impl From<ParseIntError> for SizeParseError {
    fn from(err: ParseIntError) -> Self {
        SizeParseError::ParseIntError(err)
    }
}

fn convert_size_to_bytes(size: &str) -> Result<u64, SizeParseError> {
    let re = Regex::new(r"(?P<number>[0-9]+)(?P<unit>[KMGTP]?)[Bo]?")
        .expect("Regex for parsing size is not valid");

    if let Some(capture) = re.captures(size) {
        if let Some(number_str) = capture.name("number") {
            let number: u64 = number_str.as_str().parse()?;
            if let Some(unit) = capture.name("unit") {
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
            }
        }
    }
    Err(SizeParseError::InvalidSizeFormat)
}

fn set_log_level(verbosity: u64) {
    let level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 | _ => LevelFilter::Debug,
    };

    // Initialize the logger using env_logger
    env_logger::builder().filter_level(level).init();
}

pub fn parse_cli() {
    // Define and parse the command-line arguments
    let matches = define_cli_parameters().get_matches();

    // Check if the verbose flag is present and count its occurrences
    let verbosity = matches.occurrences_of("verbose");
    set_log_level(verbosity);
}
