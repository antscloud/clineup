use crate::gps::base::GpsResolutionProvider;
use crate::gps::base::LocationInfo;
use reqwest;
struct GoogleApi {
    api_key: String,
}

impl GoogleApi {
    const URL: str = "https://maps.googleapis.com/maps/api/geocode/json";
    // Constructor to create a new GoogleApi instance with an API key
    pub fn new(api_key: String) -> Self {
        GoogleApi { api_key }
    }

    // Helper method to make the API request and process the response
    fn make_api_request(&self, lat: f32, lon: f32) -> Result<LocationInfo, String> {
        let client = reqwest::blocking::Client::builder();

        let response = client
            .get(url)
            .query(&[
                ("latlng", format!("{},{}", lat, lon)),
                ("key", self.api_key.as_str()),
            ])
            .send()
            .map_err(|err| err.to_string())?;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let json_result: serde_json::Value =
                        response.json().map_err(|err| err.to_string())?;

                    // Here, you need to extract the city, street, and country information
                    // from the JSON response and create a LocationInfo struct accordingly.
                    // Replace the dummy values below with the actual data from the API response.

                    let city = Some("New York".to_string());
                    let street = Some("123 Main St".to_string());
                    let country = Some("United States".to_string());

                    Ok(LocationInfo {
                        city,
                        street,
                        country,
                    })
                } else {
                    Err(format!("API request failed: {}", response.status()))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

impl GpsResolutionProvider for GoogleApi {
    fn get_location(&self, lat: f64, lon: f64) -> Result<LocationInfo, String> {
        let url = format!(
            "https://maps.googleapis.com/maps/api/geocode/json?latlng={},{}&key={}",
            lat, lon, self.api_key
        );

        self.make_api_request(&url)
    }
}
