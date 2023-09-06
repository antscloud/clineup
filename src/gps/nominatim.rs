use crate::errors::ClineupError;
use crate::gps::base::GpsResolutionProvider;
use reqwest;
use serde_json;
use std::cell::Cell;
use std::{thread, time};

use super::location::LocationInfo;
// Implementation for Nominatim API

pub struct Nominatim {
    email: String,
    _last_time_called: Cell<Option<time::Instant>>,
}

/// Ressource: https://nominatim.org/release-docs/latest/api/Reverse/#reverse-geocoding
impl Nominatim {
    const URL: &str = "https://nominatim.openstreetmap.org/reverse";
    pub fn new(email: String) -> Self {
        Nominatim {
            email,
            _last_time_called: Cell::new(None),
        }
    }

    fn ensure_time_gap(&self) {
        if self._last_time_called.get().is_none() {
            self._last_time_called.set(Some(time::Instant::now()));
        }

        let one_half_second = time::Duration::from_millis(1100);

        if let Some(last_time_called) = self._last_time_called.get() {
            if last_time_called.elapsed() < one_half_second {
                let to_sleep = one_half_second - last_time_called.elapsed();
                thread::sleep(to_sleep);
            }
        }
    }

    fn update_last_time_called(&self) {
        self._last_time_called.set(Some(time::Instant::now()));
    }

    pub fn make_api_request(&self, lat: f32, lon: f32) -> Result<LocationInfo, ClineupError> {
        self.ensure_time_gap();

        let client = reqwest::blocking::Client::new();

        let response = client
            .get(Self::URL)
            .query(&[
                ("lat", lat.to_string()),
                ("lon", lon.to_string()),
                ("zoom", 10.to_string()),
                ("format", "json".to_string()),
            ])
            .header("User-Agent", self.email.clone())
            .send()?;

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

impl GpsResolutionProvider for Nominatim {
    fn get_location(&self, lat: f32, lon: f32) -> Result<LocationInfo, ClineupError> {
        self.make_api_request(lat, lon)
    }
}
