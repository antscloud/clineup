// Trait for GPS resolution providers
pub trait GpsResolutionProvider {
    fn get_location(&self, lat: f32, lon: f32) -> Result<LocationInfo, String>;
}

// Struct to hold location information
#[derive(Debug)]
pub struct LocationInfo {
    country: Option<String>,
    state: Option<String>,
    county: Option<String>,
    municipality: Option<String>,
    city: Option<String>,
}

impl LocationInfo {
    pub fn new(
        country: Option<String>,
        state: Option<String>,
        county: Option<String>,
        municipality: Option<String>,
        city: Option<String>,
    ) -> Self {
        LocationInfo {
            country,
            state,
            county,
            municipality,
            city,
        }
    }

    pub fn country(&self) -> Option<&str> {
        self.country.as_deref()
    }
    pub fn state(&self) -> Option<&str> {
        self.state.as_deref()
    }
    pub fn county(&self) -> Option<&str> {
        self.county.as_deref()
    }
    pub fn municipality(&self) -> Option<&str> {
        self.municipality.as_deref()
    }
    pub fn city(&self) -> Option<&str> {
        self.city.as_deref()
    }
}
