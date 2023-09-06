use crate::errors::ClineupError;
use crate::gps::base::GpsResolutionProvider;
use crate::gps::base::LocationInfo;
use reqwest;
struct MapQuest {
    api_key: String,
}
/// Ressource: https://developer.mapquest.com/documentation/geocoding-api/reverse/get/
impl MapQuest {
    const URL: str = "https://www.mapquestapi.com/geocoding/v1/reverse";
    pub fn new(api_key: String) -> Self {
        MapQuest { api_key }
    }

    fn make_api_request(&self, lat: f32, lon: f32) -> Result<LocationInfo, ClineupError> {
        unimplemented!()
    }
}

impl GpsResolutionProvider for MapQuest {
    fn get_location(&self, lat: f64, lon: f64) -> Result<LocationInfo, ClineupError> {
        self.make_api_request(&url)
    }
}
