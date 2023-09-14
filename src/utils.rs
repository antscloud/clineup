use std::{collections::HashMap, process::exit};

use log::error;

use crate::{errors::ClineupError, placeholders::Placeholder};

pub fn is_there_a_exif_placeholder(
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) -> bool {
    let mut is_there_a_date_placeholder = false;
    for (_, placeholders) in placeholders {
        for (_, placeholder) in placeholders {
            if placeholder.is_exif_related() {
                is_there_a_date_placeholder = true;
                break;
            }
        }
    }
    is_there_a_date_placeholder
}

pub fn is_there_a_metadata_placeholder(
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) -> bool {
    let mut is_there_a_metadata_placeholder = false;
    for (_, placeholders) in placeholders {
        for (_, placeholder) in placeholders {
            if placeholder.is_os_related() {
                is_there_a_metadata_placeholder = true;
                break;
            }
        }
    }
    is_there_a_metadata_placeholder
}

pub fn is_there_a_location_placeholder(
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) -> bool {
    let mut is_there_a_location_placeholder = false;
    for (_, placeholders) in placeholders {
        for (_, placeholder) in placeholders {
            if placeholder.is_location_related() {
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
