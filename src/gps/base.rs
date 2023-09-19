use crate::errors::ClineupError;

use super::location::LocationInfo;

// Trait for GPS resolution providers
pub trait GpsResolutionProvider {
    fn get_location(&self, lat: f32, lon: f32) -> Result<LocationInfo, ClineupError>;
}
