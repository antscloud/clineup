use log::debug;

use crate::errors::ClineupError;
use crate::exif_extractor::ExifExtractor;
use crate::gps;
use crate::gps::base::GpsResolutionProvider;
use crate::gps::location::LocationInfo;

use crate::placeholders::Placeholder;
use std::cell::Cell;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::utils::is_there_a_date_placeholder;
use crate::utils::is_there_a_location_placeholder;

pub fn get_fallback_name(which: &str) -> String {
    format!("Unknown {}", which)
}
// Define a custom key type that wraps the (f32, f32) tuple
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct StringLatLon(String, String);

fn round_float_to_nth_decimal_place(num: f32, decimal_places: u32) -> f32 {
    let multiplier = 10_f32.powi(decimal_places as i32);
    (num * multiplier).round() / multiplier
}

pub struct PathFormatter<'a, 'b> {
    path_to_format: &'a String,
    placeholders: &'b HashMap<String, HashMap<String, Placeholder>>,
    reverse_geocoding: Option<Box<dyn GpsResolutionProvider>>,
    gps_positions: HashMap<StringLatLon, LocationInfo>,
    optimize_gps: bool,
}

impl<'a, 'b> PathFormatter<'a, 'b> {
    pub fn new(
        path_to_format: &'a String,
        placeholders: &'b HashMap<String, HashMap<String, Placeholder>>,
        reverse_geocoding: Option<Box<dyn GpsResolutionProvider>>,
        optimize_gps: bool,
    ) -> Self {
        let gps_positions = HashMap::new();
        PathFormatter {
            path_to_format,
            placeholders,
            reverse_geocoding,
            gps_positions,
            optimize_gps,
        }
    }

    fn get_location_info(
        &mut self,
        exif_extractor: &ExifExtractor,
    ) -> Result<LocationInfo, ClineupError> {
        if !is_there_a_location_placeholder(&self.placeholders) {
            return Err(ClineupError::NoLocationPlaceholderFound);
        }

        let lat = exif_extractor.get_latitude();
        let lon = exif_extractor.get_longitude();

        if !(lat.is_ok() && lon.is_ok()) {
            return Err(ClineupError::LatOrLonMissing);
        }

        let lat = lat.unwrap();
        let lon = lon.unwrap();

        if !self.optimize_gps {
            return self
                .reverse_geocoding
                .as_ref()
                .unwrap()
                .get_location(lat, lon);
        }

        let rounded_lat = round_float_to_nth_decimal_place(lat, 2);
        let rounded_lon = round_float_to_nth_decimal_place(lon, 2);
        let string_lat_lon = StringLatLon(rounded_lat.to_string(), rounded_lon.to_string());

        if self.gps_positions.contains_key(&string_lat_lon) {
            debug!("Get already computed location {:?}", string_lat_lon);
            return Ok(self.gps_positions.get(&string_lat_lon).unwrap().clone());
        }

        let location = self
            .reverse_geocoding
            .as_ref()
            .unwrap()
            .get_location(rounded_lat, rounded_lon);
        match location {
            Ok(v) => {
                debug!("Store location {:?}", v);
                self.gps_positions.insert(string_lat_lon, v.clone());
                return Ok(v);
            }
            Err(_) => return Err(ClineupError::LatOrLonMissing),
        };
    }

    fn get_modification_date(
        &self,
        exif_extractor: &ExifExtractor,
    ) -> Result<chrono::NaiveDateTime, ClineupError> {
        let modification_date = if is_there_a_date_placeholder(&self.placeholders) {
            exif_extractor.get_modification_date()
        } else {
            Err(ClineupError::NoLocationPlaceholderFound)
        };
        modification_date
    }
}

impl<'a, 'b> PathFormatter<'a, 'b> {
    pub fn get_formatted_path(&mut self, path: &PathBuf) -> Result<String, ClineupError> {
        let mut formatted_path = String::from(self.path_to_format.clone());
        let exif_extractor = ExifExtractor::new(path.to_string_lossy().to_string())?;

        let location = self.get_location_info(&exif_extractor);

        let modification_date = self.get_modification_date(&exif_extractor);

        for (full_text, placeholders) in self.placeholders {
            let mut result = String::new();
            let mut is_fallback = false;

            for (placeholder_text, placeholder) in placeholders {
                let current_result = match placeholder {
                    Placeholder::Year => match modification_date {
                        Ok(v) => v.format("%Y").to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Year")
                        }
                    },
                    Placeholder::Month => match modification_date {
                        Ok(v) => v.format("%m").to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Month")
                        }
                    },
                    Placeholder::Day => match modification_date {
                        Ok(v) => v.format("%d").to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Day")
                        }
                    },
                    Placeholder::Width => match exif_extractor.get_width() {
                        Ok(v) => v.to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Width")
                        }
                    },
                    Placeholder::Height => match exif_extractor.get_height() {
                        Ok(v) => v.to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Height")
                        }
                    },
                    Placeholder::CameraModel => match exif_extractor.get_camera_model() {
                        Ok(v) => v.to_string(),
                        Err(_) => {
                            is_fallback = true;
                            "Unknown Camera Model".to_string()
                        }
                    },
                    Placeholder::CameraBrand => match exif_extractor.get_camera_brand() {
                        Ok(v) => v.to_string(),
                        Err(_) => {
                            is_fallback = true;
                            "Unknown Camera Brand".to_string()
                        }
                    },
                    Placeholder::Country => match location.as_ref() {
                        Ok(v) => v
                            .country()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                is_fallback = true;
                                get_fallback_name("Country")
                            })
                            .to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Country")
                        }
                    },
                    Placeholder::State => match location.as_ref() {
                        Ok(v) => v
                            .state()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                is_fallback = true;
                                get_fallback_name("State")
                            })
                            .to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("State")
                        }
                    },
                    Placeholder::City => match location.as_ref() {
                        Ok(v) => v
                            .city()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                is_fallback = true;
                                get_fallback_name("City")
                            })
                            .to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("City")
                        }
                    },
                    Placeholder::County => match location.as_ref() {
                        Ok(v) => v.county().map(|m| m.to_string()).unwrap_or_else(|| {
                            is_fallback = true;
                            get_fallback_name("County")
                        }),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("County")
                        }
                    },
                    Placeholder::Municipality => match location.as_ref() {
                        Ok(v) => v.municipality().map(|m| m.to_string()).unwrap_or_else(|| {
                            is_fallback = true;
                            get_fallback_name("Municipality")
                        }),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Municipality")
                        }
                    },
                    Placeholder::OriginalFilename => {
                        if let Some(_path) = path.file_name() {
                            _path.to_string_lossy().to_string()
                        } else {
                            {
                                is_fallback = true;
                                get_fallback_name("Original Filename")
                            }
                        }
                    }
                    Placeholder::OriginalFolder => {
                        if let Some(_path) = path.parent() {
                            _path.to_string_lossy().to_string()
                        } else {
                            {
                                is_fallback = true;
                                get_fallback_name("Original Folder")
                            }
                        }
                    }
                    Placeholder::Fallback => placeholder_text.clone(),
                    Placeholder::Unknown => continue,
                };
                result = current_result.clone();
                if !is_fallback {
                    break;
                }
            }
            formatted_path = formatted_path.replace(full_text, result.as_str());
        }
        Ok(formatted_path)
    }
}
