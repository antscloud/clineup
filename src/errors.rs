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

    #[error("Exif error: {0}")]
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
}

impl From<exif::Error> for ClineupError {
    fn from(source: exif::Error) -> Self {
        ClineupError::ExifError {
            source,
            file: String::new(),
        }
    }
}
