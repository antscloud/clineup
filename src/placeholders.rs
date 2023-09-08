use serde::{Deserialize, Serialize};

// Configuration struct for the photo organizer
#[derive(Debug, Deserialize, Serialize)]
pub enum Placeholder {
    Year,
    Month,
    Day,
    Width,
    Height,
    CameraModel,
    CameraBrand,
    Country,
    State,
    County,
    Municipality,
    City,
    OriginalFolder,
    OriginalFilename,
    Unknown,
    Fallback,
}

impl Placeholder {
    pub fn from_string<S: Into<String>>(chain: S) -> Placeholder {
        let format_string = chain.into();
        let format_string_str = format_string.as_str();
        match format_string_str {
            "%year" => Placeholder::Year,
            "%month" => Placeholder::Month,
            "%day" => Placeholder::Day,
            "%width" => Placeholder::Width,
            "%height" => Placeholder::Height,
            "%camera_model" => Placeholder::CameraModel,
            "%camera_brand" => Placeholder::CameraBrand,
            "%country" => Placeholder::Country,
            "%state" => Placeholder::State,
            "%county" => Placeholder::County,
            "%municipality" => Placeholder::Municipality,
            "%city" => Placeholder::City,
            "%original_folder" => Placeholder::OriginalFolder,
            "%original_filename" => Placeholder::OriginalFilename,
            _ if format_string.starts_with("%") => Placeholder::Unknown,
            _ => Placeholder::Fallback,
        }
    }
}
