use clineup::cli::check_cli_config_from_placeholders;
use clineup::cli::get_cli_config;
use clineup::cli::init_logger;
use clineup::cli::parse_cli;
use clineup::path::duplicates_finder::DuplicatesFinder;
use clineup::path::formatter::PathFormatter;
use clineup::path::iterator::FileIterator;
use clineup::path::parser::map_placeholders_to_enums;
use clineup::path::parser::parse_placeholders;
use clineup::utils::get_full_format_path;
use clineup::utils::get_organization_strategy;
use clineup::utils::get_reverse_geocoding;
use indicatif::ProgressBar;
use log::debug;
use log::error;
use log::info;
use std::path::Path;
use std::process::exit;

fn main() {
    let matches = parse_cli();
    let verbosity = matches.occurrences_of("verbose");

    if matches.is_present("dry-run") && verbosity == 0 {
        init_logger(1);
    } else {
        init_logger(verbosity);
    }
    debug!("Get configuration");
    let config = get_cli_config(matches);

    let strategy = get_organization_strategy(config.strategy.as_ref());

    if strategy.is_none() {
        println!("Strategy is not set");
        exit(1)
    } else {
        debug!("Get strategy {:?}", config.strategy);
    }

    debug!("Get reverse geocoding strategy");
    let reverse_geocoding = get_reverse_geocoding(&config);

    let full_path = get_full_format_path(
        config.folder_format.as_ref(),
        config.filename_format.as_ref(),
    );

    if full_path.is_none() {
        println!("You should provide at least one of the folder or filename format.");
        exit(1);
    } else {
        debug!("Full path {:?}", full_path);
    }

    let destination = Path::new(&config.destination);

    debug!("Parsing placeholders");
    let _placeholders = parse_placeholders(full_path.as_ref().unwrap());
    let placeholders = map_placeholders_to_enums(&_placeholders);
    check_cli_config_from_placeholders(&config, &placeholders);

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

    let mut file_processed_count = 0;

    let bar = ProgressBar::new_spinner();
    bar.set_message("Start organizing files...");

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
            if file_processed_count >= config.dry_run_number_of_files {
                exit(0);
            }
            info!("{:?} -> {}", entry, good_formatted_path.display());
            file_processed_count += 1;
            continue;
        }

        strategy
            .as_ref()
            .unwrap()
            .organize(&entry, &good_formatted_path);

        file_processed_count += 1;

        bar.set_message(format!("{:?} file(s) processed", file_processed_count));
        bar.tick();
    }
    bar.finish_with_message(format!(
        "Done. {:?} file(s) processed.",
        file_processed_count
    ));
}
