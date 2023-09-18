#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::NaiveDateTime;
    use clineup::exif_extractor::ExifExtractor;

    #[test]
    fn test_instantiate_good_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/Paris-20230304-duplicated.jpg");
        let entry = ExifExtractor::new(&path);

        assert!(entry.is_ok());
    }
    #[test]
    fn test_instantiate_bad_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/empty.txt");
        let entry = ExifExtractor::new(&path);

        assert!(entry.is_err());
    }
    #[test]
    fn test_get_string_latitude() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/Paris-20230304.jpg");
        let latitude = ExifExtractor::new(&path)
            .unwrap()
            .get_string_value(exif::Tag::GPSLatitude);

        assert!(latitude.is_ok());
        assert!(latitude.unwrap() == "48 deg 51 min 29.8 sec");
    }

    #[test]
    fn test_get_float_latitude() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/Paris-20230304.jpg");
        let latitude = ExifExtractor::new(&path).unwrap().get_latitude();

        assert!(latitude.is_ok());
        assert!((latitude.unwrap() - 48.86).abs() <= 0.001);
    }

    #[test]
    fn test_get_float_longitude() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/Paris-20230304.jpg");
        let longitude = ExifExtractor::new(&path).unwrap().get_longitude();

        assert!(longitude.is_ok());
        assert!((longitude.unwrap() - 2.29).abs() <= 0.001);
    }
    #[test]
    fn test_get_exif_date() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/Paris-20230304.jpg");
        let date = ExifExtractor::new(&path).unwrap().get_exif_date();
        let expected_date =
            NaiveDateTime::parse_from_str("2023:03:04 00:00:00", "%Y:%m:%d %H:%M:%S").unwrap();

        assert!(date.is_ok());
        assert!(date.unwrap() == expected_date);
    }
    #[test]
    fn test_get_camera_model() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/Paris-20230304.jpg");
        let model = ExifExtractor::new(&path).unwrap().get_camera_model();
        let expected_brand = "rusttest";

        assert!(model.is_ok());
        assert!(model.unwrap() == expected_brand);
    }
}
