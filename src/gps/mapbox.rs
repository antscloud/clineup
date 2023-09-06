use crate::errors::ClineupError;
use crate::gps::base::GpsResolutionProvider;
use crate::gps::base::LocationInfo;
use reqwest;
struct MapBoxApi {
    api_key: String,
}

/// Ressource : https://docs.mapbox.com/api/search/geocoding/#reverse-geocoding
impl MapBoxApi {
    const URL: str = "https://api.mapbox.com/geocoding/v5/{endpoint}/{longitude},{latitude}.json";
    pub fn new(api_key: String) -> Self {
        MapBoxApi { api_key }
    }

    fn make_api_request(&self, lat: f32, lon: f32) -> Result<LocationInfo, ClineupError> {
        unimplemented!()
    }
}

impl GpsResolutionProvider for MapBoxApi {
    fn get_location(&self, lat: f64, lon: f64) -> Result<LocationInfo, ClineupError> {
        self.make_api_request(&url)
    }
}
