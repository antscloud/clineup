use super::location::LocationInfo;
use crate::errors::ClineupError;
use crate::gps::base::GpsResolutionProvider;
use reqwest;
use serde_json;
use std::cell::Cell;
use std::{thread, time};
// Trait for nomitatim request
// This is use for testing
// We can create a fake trait to not depend on nominatim provider
pub trait BlockingProvider {
    fn get_response(&self, lat: f32, lon: f32)
        -> Result<reqwest::blocking::Response, ClineupError>;
}
struct NominatimProvider {
    email: String,
    url: String,
}

impl NominatimProvider {
    pub fn new(email: String, url: String) -> Self {
        NominatimProvider { email, url }
    }
}

impl BlockingProvider for NominatimProvider {
    fn get_response(
        &self,
        lat: f32,
        lon: f32,
    ) -> Result<reqwest::blocking::Response, ClineupError> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(self.url.clone())
            .query(&[
                ("lat", lat.to_string()),
                ("lon", lon.to_string()),
                ("zoom", 10.to_string()),
                ("format", "json".to_string()),
            ])
            .header("User-Agent", self.email.clone())
            .send()
            .map_err(|err| ClineupError::ReqwestError(err));
        response
    }
}

/// Struct use for API
///
/// It respects the usage policy by using a blocking request and ensures that
/// two request are at least 1.5 seconds apart
///
/// ## Ressources :
/// https://nominatim.org/release-docs/latest/api/Reverse/#reverse-geocoding
/// https://operations.osmfoundation.org/policies/nominatim/
pub struct Nominatim {
    last_time_called: Cell<Option<time::Instant>>,
    provider: Box<dyn BlockingProvider>,
}

impl Nominatim {
    pub fn from_provider(provider: Box<dyn BlockingProvider>) -> Self {
        Nominatim {
            last_time_called: Cell::new(None),
            provider,
        }
    }
    pub fn new(email: String) -> Self {
        let provider = Box::new(NominatimProvider::new(
            email,
            "https://nominatim.openstreetmap.org/reverse".to_string(),
        ));

        Nominatim {
            last_time_called: Cell::new(None),
            provider,
        }
    }
    fn ensure_time_gap(&self) {
        if self.last_time_called.get().is_none() {
            self.last_time_called.set(Some(time::Instant::now()));
        }

        let one_half_second = time::Duration::from_millis(1100);

        if let Some(last_time_called) = self.last_time_called.get() {
            if last_time_called.elapsed() < one_half_second {
                let to_sleep = one_half_second - last_time_called.elapsed();
                thread::sleep(to_sleep);
            }
        }
    }

    fn update_last_time_called(&self) {
        self.last_time_called.set(Some(time::Instant::now()));
    }
}

impl GpsResolutionProvider for Nominatim {
    fn get_location(&self, lat: f32, lon: f32) -> Result<LocationInfo, ClineupError> {
        self.ensure_time_gap();

        let response = self.provider.get_response(lat, lon)?;

        self.update_last_time_called();

        let status = response.status();

        if status.is_success() {
            let response_text = response.text()?;

            // Deserialize the JSON response using serde_json
            let json_result: serde_json::Value = serde_json::from_str(&response_text)?;

            // Check if "address" field exists and get its value
            if let Some(address) = json_result.get("address") {
                let country = address
                    .get("country")
                    .and_then(serde_json::Value::as_str)
                    .map(String::from);

                let state = address
                    .get("state")
                    .and_then(serde_json::Value::as_str)
                    .map(String::from);

                let county = address
                    .get("county")
                    .and_then(serde_json::Value::as_str)
                    .map(String::from);

                let municipality = address
                    .get("municipality")
                    .and_then(serde_json::Value::as_str)
                    .map(String::from);

                let city = address
                    .get("city")
                    .or_else(|| address.get("village"))
                    .and_then(serde_json::Value::as_str)
                    .map(String::from);

                return Ok(LocationInfo::new(
                    country,
                    state,
                    county,
                    municipality,
                    city,
                ));
            }
        }

        Err(ClineupError::HttpFailedCodeError(status.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http;

    #[test]
    fn test_get_location_success() {
        struct MockProvider {}

        impl BlockingProvider for MockProvider {
            fn get_response(
                &self,
                _lat: f32,
                _lon: f32,
            ) -> Result<reqwest::blocking::Response, ClineupError> {
                let json_response_body = r#"{"place_id":83293355,"licence":"Data © OpenStreetMap contributors, ODbL 1.0. http://osm.org/copyright","osm_type":"relation","osm_id":7444,"lat":"48.8588897","lon":"2.3200410217200766","class":"boundary","type":"administrative","place_rank":15,"importance":0.8317101715588673,"addresstype":"suburb","name":"Paris","display_name":"Paris, Île-de-France, France métropolitaine, France","address":{"suburb":"Paris","city_district":"Paris","city":"Paris","ISO3166-2-lvl6":"FR-75","state":"Île-de-France","ISO3166-2-lvl4":"FR-IDF","region":"France métropolitaine","country":"France","country_code":"fr"},"boundingbox":["48.8155755","48.9021560","2.2241220","2.4697602"]}"#;

                let http_response = http::response::Builder::new()
                    .status(http::StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(json_response_body)
                    .unwrap();

                let reqwest_response = reqwest::blocking::Response::from(http_response);

                Ok(reqwest_response)
            }
        }

        // Convert the custom ResponseBuilder to a reqwest Response
        let mock_provider = MockProvider {};

        let nominatim = Nominatim::from_provider(Box::new(mock_provider));

        let result = nominatim.get_location(0.0, 0.0);

        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().country().unwrap() == "France".to_string());
        assert!(result.as_ref().unwrap().city().unwrap() == "Paris".to_string());
        assert!(result.as_ref().unwrap().state().unwrap() == "Île-de-France".to_string());
        assert!(result.as_ref().unwrap().municipality().is_none());
        assert!(result.as_ref().unwrap().county().is_none());
        // Add more assertions based on the expected behavior
    }

    #[test]
    fn test_get_location_wrong_json_response() {
        struct MockProvider {}

        impl BlockingProvider for MockProvider {
            fn get_response(
                &self,
                _lat: f32,
                _lon: f32,
            ) -> Result<reqwest::blocking::Response, ClineupError> {
                let json_response_body = r#"{"key1": "value1", "key2": 42}"#;

                let http_response = http::response::Builder::new()
                    .status(http::StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(json_response_body)
                    .unwrap();

                let reqwest_response = reqwest::blocking::Response::from(http_response);

                Ok(reqwest_response)
            }
        }

        // Convert the custom ResponseBuilder to a reqwest Response
        let mock_provider = MockProvider {};

        let nominatim = Nominatim::from_provider(Box::new(mock_provider));

        let result = nominatim.get_location(0.0, 0.0);

        assert!(result.is_err());
        // Add more assertions based on the expected behavior
    }

    #[test]
    fn test_get_location_failure() {
        struct MockProvider {}

        impl BlockingProvider for MockProvider {
            fn get_response(
                &self,
                _lat: f32,
                _lon: f32,
            ) -> Result<reqwest::blocking::Response, ClineupError> {
                // Create a JSON response body
                let error_message = "An error occurred";
                let io_error = std::io::Error::new(std::io::ErrorKind::Other, error_message);
                Err(ClineupError::IoError(io_error))
            }
        }

        // Convert the custom ResponseBuilder to a reqwest Response
        let mock_provider = MockProvider {};

        let nominatim = Nominatim::from_provider(Box::new(mock_provider));

        let result = nominatim.get_location(0.0, 0.0);

        assert!(result.is_err());
    }
}
