use crate::exif_extractor::ExifExtractor;
use crate::gps::nominatim;
use crate::path_parser::get_placeholders_map;
use crate::placeholders::{self, Placeholder};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
#[derive(Debug, Deserialize, Serialize)]

pub struct PathFormatter {
    raw_path: String,
    placeholders_map: HashMap<String, Placeholder>,
    computed: HashMap<String, String>,
    // locations: HashMap<Tuple(float, float), LocationInfo>
}

impl PathFormatter {
    pub fn new(raw_path: String) -> Self {
        let placeholders_map = get_placeholders_map(&raw_path);
        let computed = HashMap::new();
        PathFormatter {
            raw_path,
            placeholders_map,
            computed,
        }
    }

    pub fn get_formatted_path(&self, path: String) -> Result<String, String> {
        let mut formatted_path = String::from(self.raw_path.clone());
        let _real_path = self.raw_path.clone();
        let real_path = Path::new(_real_path.as_str());
        let exif_extractor = ExifExtractor::new(path)?;
        let reverse_geocoding = nominatim::Nominatim::new();

        let location = if self.placeholders_map.iter().any(|(_, tag)| {
            matches!(
                tag,
                Placeholder::City | Placeholder::Country | Placeholder::County | Placeholder::State
            )
        }) {
            let lat = exif_extractor.get_latitude();
            let lon = exif_extractor.get_longitude();
            if lat.is_ok() && lon.is_ok() {
                let lat = lat.unwrap();
                let lon = lon.unwrap();

                reverse_geocoding.make_api_request(lat, lon)
            } else {
                Err("Latitude or longitude missing".to_string())
            }
        } else {
            Err("No location tags found in placeholders_map".to_string())
        };

        let modification_date = if self.placeholders_map.iter().any(|(_, tag)| {
            matches!(
                tag,
                Placeholder::Year | Placeholder::Month | Placeholder::Day
            )
        }) {
            exif_extractor.get_modification_date()
        } else {
            Err("No modification date tags found in placeholders_map".to_string())
        };

        for (key, tag) in &self.placeholders_map {
            let str_tag = match tag {
                Placeholder::Year => match modification_date {
                    Ok(v) => v.format("%Y").to_string(),
                    Err(_) => "Unknown Year".to_string(),
                },
                Placeholder::Month => match modification_date {
                    Ok(v) => v.format("%m").to_string(),
                    Err(_) => "Unknown Month".to_string(),
                },
                Placeholder::Day => match modification_date {
                    Ok(v) => v.format("%d").to_string(),
                    Err(_) => "Unknown Day".to_string(),
                },
                Placeholder::Width => match exif_extractor.get_width() {
                    Ok(v) => v.to_string(),
                    Err(_) => "Unknown Width".to_string(),
                },
                Placeholder::Height => match exif_extractor.get_height() {
                    Ok(v) => v.to_string(),
                    Err(_) => "Unknown Height".to_string(),
                },
                Placeholder::CameraModel => match exif_extractor.get_camera_model() {
                    Ok(v) => v.to_string(),
                    Err(_) => "Unknown Camera Model".to_string(),
                },
                Placeholder::CameraBrand => match exif_extractor.get_camera_brand() {
                    Ok(v) => v.to_string(),
                    Err(_) => "Unknown Camera Brand".to_string(),
                },
                Placeholder::Country => match location.as_ref() {
                    Ok(v) => v.country().unwrap_or_else(|| "Unknown Country").to_string(),
                    Err(_) => "Unknown Country".to_string(),
                },
                Placeholder::State => match location.as_ref() {
                    Ok(v) => v.state().unwrap_or_else(|| "Unknown State").to_string(),
                    Err(_) => "Unknown State".to_string(),
                },
                Placeholder::City => match location.as_ref() {
                    Ok(v) => v.city().unwrap_or_else(|| "Unknown City").to_string(),
                    Err(_) => "Unknown City".to_string(),
                },
                Placeholder::County => match location.as_ref() {
                    Ok(v) => v.county().unwrap_or_else(|| "Unknown County").to_string(),
                    Err(_) => "Unknown County".to_string(),
                },
                Placeholder::Municipality => match location.as_ref() {
                    Ok(v) => v
                        .municipality()
                        .unwrap_or_else(|| "Unknown Municipality")
                        .to_string(),
                    Err(_) => "Unknown Municipality".to_string(),
                },
                Placeholder::Event => "Unknown Event".to_string(),
                Placeholder::OriginalFilename => {
                    if let Some(_path) = real_path.file_name() {
                        _path.to_string_lossy().to_string()
                    } else {
                        "Unknown Original Filename".to_string()
                    }
                }
                Placeholder::OriginalFolder => {
                    if let Some(_path) = real_path.parent() {
                        _path.to_string_lossy().to_string()
                    } else {
                        "Unknown Original Folder".to_string()
                    }
                }
                Placeholder::Fallback => continue,
            };
            formatted_path = formatted_path.replace(key, str_tag.as_str());
        }
        Ok(formatted_path)
    }
}
