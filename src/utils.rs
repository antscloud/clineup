use std::{collections::HashMap, process::exit};

use log::error;

use crate::{errors::ClineupError, placeholders::Placeholder};

pub fn is_there_a_date_placeholder(
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) -> bool {
    let mut is_there_a_date_placeholder = false;
    for (_, placeholders) in placeholders {
        for (_, placeholder) in placeholders {
            if matches!(
                placeholder,
                Placeholder::Year | Placeholder::Month | Placeholder::Day
            ) {
                is_there_a_date_placeholder = true;
                break;
            }
        }
    }
    is_there_a_date_placeholder
}

pub fn is_there_a_location_placeholder(
    placeholders: &HashMap<String, HashMap<String, Placeholder>>,
) -> bool {
    let mut is_there_a_location_placeholder = false;
    for (_, placeholders) in placeholders {
        for (_, placeholder) in placeholders {
            if matches!(
                placeholder,
                Placeholder::City
                    | Placeholder::Country
                    | Placeholder::County
                    | Placeholder::State
                    | Placeholder::Municipality
            ) {
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
