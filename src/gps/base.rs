use crate::errors::ClineupError;

use super::location::LocationInfo;
use reqwest::Response;
// Trait for GPS resolution providers
pub trait GpsResolutionProvider {
    fn get_location(&self, lat: f32, lon: f32) -> Result<LocationInfo, ClineupError>;
}
