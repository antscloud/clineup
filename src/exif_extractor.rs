use exif::Exif;

pub struct ExifExtractor {
    exif: Exif,
}

impl ExifExtractor {
    pub fn new(path: String) -> Result<Self, String> {
        let fd = std::fs::File::open(&path).map_err(|err| err.to_string())?;
        let mut bufreader = std::io::BufReader::new(&fd);
        let exifreader = exif::Reader::new();
        let exif = exifreader
            .read_from_container(&mut bufreader)
            .map_err(|err| err.to_string())?;
        Ok(ExifExtractor { exif })
    }

    pub fn get_float_value(&self, tag: exif::Tag) -> Result<f32, String> {
        if let Some(field) = self.exif.get_field(tag, exif::In::PRIMARY) {
            if let exif::Value::Rational(ref v) = field.value {
                if !v.is_empty() {
                    return Ok(v.iter().map(|s| s.to_f32()).sum());
                }
            }
        }
        Err(format!("Invalid or missing tag: {:?}", tag))
    }

    pub fn get_gps_float_value(&self, tag: exif::Tag) -> Result<f32, String> {
        if let Some(field) = self.exif.get_field(tag, exif::In::PRIMARY) {
            if let exif::Value::Rational(ref v) = field.value {
                if v.len() >= 3 {
                    let dividers = [1, 60, 3600];
                    let gps_float: f32 = v
                        .iter()
                        .zip(&dividers)
                        .take(3)
                        .map(|(element, divider)| element.to_f32() / *divider as f32)
                        .sum();
                    let rounded_gps_float = (gps_float * 100.0).round() / 100.0;
                    return Ok(rounded_gps_float);
                }
            }
        }
        Err(format!("Invalid or missing tag: {:?}", tag))
    }

    pub fn get_string_value(&self, tag: exif::Tag) -> Result<String, String> {
        if let Some(field) = self.exif.get_field(tag, exif::In::PRIMARY) {
            let value = field.display_value().to_string().replace("\"", "");
            let components: Vec<&str> = value.split(',').map(str::trim).collect();
            let concatenated_value = components.join("");
            if !concatenated_value.is_empty() {
                return Ok(concatenated_value);
            }
        }
        Err(format!("Invalid or missing tag: {:?}", tag))
    }

    pub fn get_latitude(&self) -> Result<f32, String> {
        self.get_gps_float_value(exif::Tag::GPSLatitude)
    }

    pub fn get_longitude(&self) -> Result<f32, String> {
        self.get_gps_float_value(exif::Tag::GPSLongitude)
    }

    pub fn get_altitude(&self) -> Result<f32, String> {
        self.get_float_value(exif::Tag::GPSAltitude)
    }

    pub fn get_modification_date(&self) -> Result<chrono::NaiveDateTime, String> {
        let date = self
            .exif
            .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY);
        if let Some(value) = date {
            let date_string = value.display_value().to_string();
            return chrono::NaiveDateTime::parse_from_str(&date_string, "%Y-%m-%d %H:%M:%S")
                .map_err(|err| err.to_string());
        }
        Err("Date tag is missing".to_string())
    }

    pub fn get_width(&self) -> Result<f32, String> {
        self.get_float_value(exif::Tag::ImageWidth)
    }

    pub fn get_height(&self) -> Result<f32, String> {
        self.get_float_value(exif::Tag::ImageLength)
    }

    pub fn get_camera_model(&self) -> Result<String, String> {
        self.get_string_value(exif::Tag::Model)
    }

    pub fn get_camera_brand(&self) -> Result<String, String> {
        self.get_string_value(exif::Tag::Make)
    }
}
