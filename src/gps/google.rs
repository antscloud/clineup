use crate::errors::ClineupError;
use crate::gps::base::GpsResolutionProvider;
use crate::gps::base::LocationInfo;
use reqwest;
struct GoogleApi {
    api_key: String,
}
/// Ressource: https://developers.google.com/maps/documentation/geocoding/requests-reverse-geocoding
impl GoogleApi {
    const URL: str = "https://maps.googleapis.com/maps/api/geocode/json";
    pub fn new(api_key: String) -> Self {
        GoogleApi { api_key }
    }

    fn make_api_request(&self, lat: f32, lon: f32) -> Result<LocationInfo, ClineupError> {
        unimplemented!()
    }
}

impl GpsResolutionProvider for GoogleApi {
    fn get_location(&self, lat: f64, lon: f64) -> Result<LocationInfo, ClineupError> {
        self.make_api_request(&url)
    }
}
