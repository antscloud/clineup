use exif::Error as _ExifError;
use log::debug;
use log::warn;

use crate::errors::ClineupError;

use crate::exif_extractor::ExifExtractor;
use crate::gps::base::GpsResolutionProvider;
use crate::gps::location::LocationInfo;

use crate::placeholders::Placeholder;
use crate::utils::is_there_a_exif_placeholder;
use crate::utils::is_there_a_location_placeholder;
use crate::utils::is_there_a_metadata_placeholder;
use chrono::prelude::{DateTime, Local};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

pub fn get_fallback_name(which: &str) -> String {
    format!("Unknown {}", which)
}

macro_rules! handle_placeholder {
    ($provider:expr, $placeholder:expr, $fallback_name:expr, $formatter:expr, $is_fallback:ident) => {
        match $provider
            .as_ref()
            .ok_or_else(|| ClineupError::InvalidPlaceholderMapping($placeholder.to_string()))?
        {
            Ok(v) => match $formatter(v) {
                Ok(result) => result,
                Err(err) => {
                    warn!("{}", err);
                    $is_fallback = true;
                    get_fallback_name($fallback_name)
                }
            },
            Err(err) => {
                warn!("{}", err);
                $is_fallback = true;
                get_fallback_name($fallback_name)
            }
        }
    };
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
        let lat = exif_extractor.get_latitude()?;
        let lon = exif_extractor.get_longitude()?;

        if !self.optimize_gps {
            return self
                .reverse_geocoding
                .as_ref()
                .unwrap()
                .get_location(lat, lon);
        }

        let rounded_lat = round_float_to_nth_decimal_place(lat, 1);
        let rounded_lon = round_float_to_nth_decimal_place(lon, 1);
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
                Ok(v)
            }
            Err(_) => Err(ClineupError::LatOrLonMissing),
        }
    }

    fn get_file_metadata(&self, path: &PathBuf) -> Result<std::fs::Metadata, ClineupError> {
        if is_there_a_metadata_placeholder(self.placeholders) {
            std::fs::metadata(path).map_err(ClineupError::from)
        } else {
            Err(ClineupError::NoLocationPlaceholderFound)
        }
    }
}

impl<'a, 'b> PathFormatter<'a, 'b> {
    pub fn get_formatted_path(&mut self, path: &PathBuf) -> Result<PathBuf, ClineupError> {
        let mut formatted_path = self.path_to_format.clone();

        let file_metadata = if is_there_a_metadata_placeholder(self.placeholders) {
            Some(self.get_file_metadata(path))
        } else {
            None
        };

        let exif_extractor = if is_there_a_exif_placeholder(self.placeholders) {
            Some(ExifExtractor::new(path))
        } else {
            None
        };

        let location = if is_there_a_location_placeholder(self.placeholders) {
            if let Some(result) = &exif_extractor {
                match result {
                    Ok(v) => Some(self.get_location_info(v)),
                    Err(_) => Some(Err(ClineupError::ExifError {
                        source: _ExifError::NotFound("No exif data found"),
                        file: path.to_string_lossy().to_string(),
                    })),
                }
            } else {
                None
            }
        } else {
            None
        };

        for (full_text, placeholders) in self.placeholders {
            let mut result = String::new();
            let mut is_fallback = false;

            for (placeholder_text, placeholder) in placeholders {
                let current_result = match placeholder {
                    Placeholder::Year => {
                        handle_placeholder!(
                            exif_extractor,
                            "Year",
                            "Year",
                            |v: &ExifExtractor| {
                                v.get_exif_date().map(|date| date.format("%Y").to_string())
                            },
                            is_fallback
                        )
                    }
                    Placeholder::Month => {
                        handle_placeholder!(
                            exif_extractor,
                            "Month",
                            "Month",
                            |v: &ExifExtractor| {
                                v.get_exif_date().map(|date| date.format("%m").to_string())
                            },
                            is_fallback
                        )
                    }
                    Placeholder::Day => {
                        handle_placeholder!(
                            exif_extractor,
                            "Day",
                            "Day",
                            |v: &ExifExtractor| {
                                v.get_exif_date().map(|date| date.format("%d").to_string())
                            },
                            is_fallback
                        )
                    }
                    Placeholder::Width => {
                        handle_placeholder!(
                            exif_extractor,
                            "Width",
                            "Width",
                            |v: &ExifExtractor| { v.get_width().map(|width| width.to_string()) },
                            is_fallback
                        )
                    }
                    Placeholder::Height => {
                        handle_placeholder!(
                            exif_extractor,
                            "Height",
                            "Height",
                            |v: &ExifExtractor| { v.get_height().map(|height| height.to_string()) },
                            is_fallback
                        )
                    }
                    Placeholder::CameraModel => {
                        handle_placeholder!(
                            exif_extractor,
                            "Camera Model",
                            "Camera Model",
                            |v: &ExifExtractor| {
                                v.get_camera_model().map(|model| model.to_string())
                            },
                            is_fallback
                        )
                    }
                    Placeholder::CameraBrand => {
                        handle_placeholder!(
                            exif_extractor,
                            "Camera Brand",
                            "Camera Brand",
                            |v: &ExifExtractor| {
                                v.get_camera_brand().map(|brand| brand.to_string())
                            },
                            is_fallback
                        )
                    }
                    Placeholder::CTimeYear => handle_placeholder!(
                        file_metadata.as_ref(),
                        "CTimeYear",
                        "Creation Time Year",
                        |v: &std::fs::Metadata| {
                            v.created()
                                .map(|date| DateTime::<Local>::from(date).format("%Y").to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::CTimeMonth => handle_placeholder!(
                        file_metadata.as_ref(),
                        "CTimeMonth",
                        "Creation Time Month",
                        |v: &std::fs::Metadata| {
                            v.clone()
                                .created()
                                .map(|date| DateTime::<Local>::from(date).format("%m").to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::CTimeDay => handle_placeholder!(
                        file_metadata.as_ref(),
                        "CTimeDay",
                        "Creation Time Day",
                        |v: &std::fs::Metadata| {
                            v.clone()
                                .created()
                                .map(|date| DateTime::<Local>::from(date).format("%d").to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::MTimeYear => handle_placeholder!(
                        file_metadata.as_ref(),
                        "MTimeYear",
                        "Modification Time Year",
                        |v: &std::fs::Metadata| {
                            v.modified()
                                .map(|date| DateTime::<Local>::from(date).format("%Y").to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::MTimeMonth => handle_placeholder!(
                        file_metadata.as_ref(),
                        "MTimeMonth",
                        "Modification Time Month",
                        |v: &std::fs::Metadata| {
                            v.modified()
                                .map(|date| DateTime::<Local>::from(date).format("%m").to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::MTimeDay => handle_placeholder!(
                        file_metadata.as_ref(),
                        "MTimeDay",
                        "Modification Time Day",
                        |v: &std::fs::Metadata| {
                            v.modified()
                                .map(|date| DateTime::<Local>::from(date).format("%d").to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::Country => handle_placeholder!(
                        location.as_ref(),
                        "Country",
                        "Country",
                        |v: &LocationInfo| {
                            v.country()
                                .ok_or(ClineupError::MissingLocation("Country".to_string()))
                                .map(|v| v.to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::State => handle_placeholder!(
                        location.as_ref(),
                        "State",
                        "State",
                        |v: &LocationInfo| {
                            v.state()
                                .ok_or(ClineupError::MissingLocation("Country".to_string()))
                                .map(|v| v.to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::City => handle_placeholder!(
                        location.as_ref(),
                        "City",
                        "City",
                        |v: &LocationInfo| {
                            v.city()
                                .ok_or(ClineupError::MissingLocation("City".to_string()))
                                .map(|v| v.to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::County => handle_placeholder!(
                        location.as_ref(),
                        "County",
                        "County",
                        |v: &LocationInfo| {
                            v.county()
                                .ok_or(ClineupError::MissingLocation("County".to_string()))
                                .map(|v| v.to_string())
                        },
                        is_fallback
                    ),
                    Placeholder::Municipality => handle_placeholder!(
                        location.as_ref(),
                        "Municipality",
                        "Municipality",
                        |v: &LocationInfo| {
                            v.municipality()
                                .ok_or(ClineupError::MissingLocation("Municipality".to_string()))
                                .map(|v| v.to_string())
                        },
                        is_fallback
                    ),
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
                    Placeholder::Unknown => placeholder_text.clone(),
                };

                result = current_result.clone();
                if !is_fallback {
                    break;
                }
            }
            formatted_path = formatted_path.replace(full_text, result.as_str());
        }
        Ok(Path::new(&formatted_path).to_path_buf())
    }
}
