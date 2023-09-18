use log::debug;

use crate::errors::ClineupError;
use crate::exif_extractor::ExifExtractor;
use crate::gps;
use crate::gps::base::GpsResolutionProvider;
use crate::gps::location::LocationInfo;

use crate::placeholders::Placeholder;
use crate::utils::is_there_a_exif_placeholder;
use crate::utils::is_there_a_location_placeholder;
use crate::utils::is_there_a_metadata_placeholder;
use chrono::prelude::{DateTime, Local};
use std::cell::Cell;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

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
        let lat = exif_extractor.get_latitude()?.clone();
        let lon = exif_extractor.get_longitude()?.clone();

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
                return Ok(v);
            }
            Err(_) => return Err(ClineupError::LatOrLonMissing),
        };
    }

    fn get_file_metadata(&self, path: &PathBuf) -> Result<std::fs::Metadata, ClineupError> {
        let metadata = if is_there_a_metadata_placeholder(&self.placeholders) {
            std::fs::metadata(path).map_err(|e| ClineupError::from(e))
        } else {
            Err(ClineupError::NoLocationPlaceholderFound)
        };
        metadata
    }
}

impl<'a, 'b> PathFormatter<'a, 'b> {
    pub fn get_formatted_path(&mut self, path: &PathBuf) -> Result<PathBuf, ClineupError> {
        let mut formatted_path = String::from(self.path_to_format.clone());

        let file_metadata = if is_there_a_metadata_placeholder(self.placeholders) {
            let _file_metadata = self.get_file_metadata(path);
            match _file_metadata {
                Ok(v) => Some(v),
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        let exif_extractor = if is_there_a_exif_placeholder(self.placeholders) {
            let _exif_extractor = ExifExtractor::new(path);
            match _exif_extractor {
                Ok(v) => Some(v),
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        let location = if is_there_a_location_placeholder(self.placeholders) {
            let _location = self.get_location_info(exif_extractor.as_ref().unwrap());
            match _location {
                Ok(v) => Some(v),
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        for (full_text, placeholders) in self.placeholders {
            let mut result = String::new();
            let mut is_fallback = false;

            for (placeholder_text, placeholder) in placeholders {
                let current_result = match placeholder {
                    Placeholder::Year => match exif_extractor
                        .as_ref()
                        .ok_or_else(|| ClineupError::InvalidPlaceholderMapping("Year".to_string()))?
                        .get_exif_date()
                    {
                        Ok(v) => v.format("%Y").to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Year")
                        }
                    },
                    Placeholder::Month => match exif_extractor
                        .as_ref()
                        .ok_or_else(|| {
                            ClineupError::InvalidPlaceholderMapping("Month".to_string())
                        })?
                        .get_exif_date()
                    {
                        Ok(v) => v.format("%m").to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Month")
                        }
                    },
                    Placeholder::Day => match exif_extractor
                        .as_ref()
                        .ok_or_else(|| ClineupError::InvalidPlaceholderMapping("Day".to_string()))?
                        .get_exif_date()
                    {
                        Ok(v) => v.format("%d").to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Day")
                        }
                    },
                    Placeholder::Width => match exif_extractor
                        .as_ref()
                        .ok_or_else(|| {
                            ClineupError::InvalidPlaceholderMapping("Width".to_string())
                        })?
                        .get_exif_date()
                    {
                        Ok(v) => v.to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Width")
                        }
                    },
                    Placeholder::Height => match exif_extractor
                        .as_ref()
                        .ok_or_else(|| {
                            ClineupError::InvalidPlaceholderMapping("Height".to_string())
                        })?
                        .get_exif_date()
                    {
                        Ok(v) => v.to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Height")
                        }
                    },
                    Placeholder::CameraModel => match exif_extractor
                        .as_ref()
                        .ok_or_else(|| {
                            ClineupError::InvalidPlaceholderMapping("Model".to_string())
                        })?
                        .get_camera_model()
                    {
                        Ok(v) => v.to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Camera Model")
                        }
                    },
                    Placeholder::CameraBrand => match exif_extractor
                        .as_ref()
                        .ok_or_else(|| ClineupError::InvalidPlaceholderMapping("Make".to_string()))?
                        .get_camera_brand()
                    {
                        Ok(v) => v.to_string(),
                        Err(_) => {
                            is_fallback = true;
                            get_fallback_name("Camera Brand")
                        }
                    },
                    Placeholder::CTimeYear => match &file_metadata {
                        Some(v) => {
                            let sys_modified = v.clone().created()?;
                            let datetime = DateTime::<Local>::from(sys_modified);
                            datetime.format("%Y").to_string()
                        }
                        None => {
                            is_fallback = true;
                            get_fallback_name("Creation Time Year")
                        }
                    },
                    Placeholder::CTimeMonth => match &file_metadata {
                        Some(v) => {
                            let sys_modified = v.clone().created()?;
                            let datetime = DateTime::<Local>::from(sys_modified);
                            datetime.format("%m").to_string()
                        }
                        None => {
                            is_fallback = true;
                            get_fallback_name("Creation Time Month")
                        }
                    },
                    Placeholder::CTimeDay => match &file_metadata {
                        Some(v) => {
                            let sys_modified = v.clone().created()?;
                            let datetime = DateTime::<Local>::from(sys_modified);
                            datetime.format("%d").to_string()
                        }
                        None => {
                            is_fallback = true;
                            get_fallback_name("Creation Time Day")
                        }
                    },
                    Placeholder::MTimeYear => match &file_metadata {
                        Some(v) => {
                            let sys_modified = v.clone().modified()?;
                            let datetime = DateTime::<Local>::from(sys_modified);
                            datetime.format("%Y").to_string()
                        }
                        None => {
                            is_fallback = true;
                            get_fallback_name("Modification Time Year")
                        }
                    },
                    Placeholder::MTimeMonth => match &file_metadata {
                        Some(v) => {
                            let sys_modified = v.clone().modified()?;
                            let datetime = DateTime::<Local>::from(sys_modified);
                            datetime.format("%m").to_string()
                        }
                        None => {
                            is_fallback = true;
                            get_fallback_name("Modification Time Month")
                        }
                    },
                    Placeholder::MTimeDay => match &file_metadata {
                        Some(v) => {
                            let sys_modified = v.clone().modified()?;
                            let datetime = DateTime::<Local>::from(sys_modified);
                            datetime.format("%d").to_string()
                        }
                        None => {
                            is_fallback = true;
                            get_fallback_name("Modification Time Day")
                        }
                    },
                    Placeholder::Country => match location.as_ref() {
                        Some(v) => v
                            .country()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                is_fallback = true;
                                get_fallback_name("Country")
                            })
                            .to_string(),
                        None => {
                            is_fallback = true;
                            get_fallback_name("Country")
                        }
                    },
                    Placeholder::State => match location.as_ref() {
                        Some(v) => v
                            .state()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                is_fallback = true;
                                get_fallback_name("State")
                            })
                            .to_string(),
                        None => {
                            is_fallback = true;
                            get_fallback_name("State")
                        }
                    },
                    Placeholder::City => match location.as_ref() {
                        Some(v) => v
                            .city()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| {
                                is_fallback = true;
                                get_fallback_name("City")
                            })
                            .to_string(),
                        None => {
                            is_fallback = true;
                            get_fallback_name("City")
                        }
                    },
                    Placeholder::County => match location.as_ref() {
                        Some(v) => v.county().map(|m| m.to_string()).unwrap_or_else(|| {
                            is_fallback = true;
                            get_fallback_name("County")
                        }),
                        None => {
                            is_fallback = true;
                            get_fallback_name("County")
                        }
                    },
                    Placeholder::Municipality => match location.as_ref() {
                        Some(v) => v.municipality().map(|m| m.to_string()).unwrap_or_else(|| {
                            is_fallback = true;
                            get_fallback_name("Municipality")
                        }),
                        None => {
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
