use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum ClineupError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Exif error for file {file}: {source}")]
    ExifError { source: exif::Error, file: String },

    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Serde Json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Date time parsing error: {0}")]
    DateTimeParseError(#[from] chrono::ParseError),

    #[error("Missing EXIF tag: {tag}")]
    ExifMissingTag { tag: String },

    #[error("Request fails with code: {0}")]
    HttpFailedCodeError(String),

    #[error("Hash error: {0}")]
    HashError(String),

    #[error("Latitude or longitude missing")]
    LatOrLonMissing,

    #[error("There is no date placeholder")]
    NoDatePlaceholderFound,

    #[error("There is no location placeholder")]
    NoLocationPlaceholderFound,

    #[error("Invalid size format : {0}")]
    InvalidSizeFormat(String),

    #[error("Invalid regex : {0}")]
    RegexError(#[from] regex::Error),

    #[error("Invalid number format: {0}")]
    InvalidNumberFormat(String),

    #[error("Invalid geocoding provider: {0}")]
    InvalidGeocodingProvider(String),

    #[error("Invalid organization strategy: {0}")]
    InvalidOrganization(String),
}

impl From<exif::Error> for ClineupError {
    fn from(source: exif::Error) -> Self {
        ClineupError::ExifError {
            source,
            file: String::new(),
        }
    }
}
