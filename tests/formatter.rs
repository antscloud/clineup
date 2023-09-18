#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clineup::path::formatter::PathFormatter;
    use clineup::path::parser::{map_placeholders_to_enums, parse_placeholders};

    #[test]
    fn test_instantiate() {
        let path_to_format = "tests/data/output/{%year}/{%camera_brand|camera_model}".to_string();
        let _placeholders = parse_placeholders(&path_to_format);
        let placeholders = map_placeholders_to_enums(&_placeholders);
        PathFormatter::new(&path_to_format, &placeholders, None, false);
    }
    #[test]
    fn test_get_formatted_path() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/Paris-20230304-duplicated.jpg");
        let path_to_format =
            "tests/data/output/{%typo}_{%year}/{%camera_brand|%camera_model}/{%camera_brand|fallback}"
                .to_string();
        let _placeholders = parse_placeholders(&path_to_format);
        let placeholders = map_placeholders_to_enums(&_placeholders);
        let mut path_formatter = PathFormatter::new(&path_to_format, &placeholders, None, false);
        let formatted_path = path_formatter.get_formatted_path(&path);
        let expected_path = PathBuf::from("tests/data/output/%typo_2023/rusttest/fallback");
        assert!(formatted_path.is_ok());
        assert!(formatted_path.unwrap() == expected_path);
    }
    #[test]
    fn test_get_formatted_path_empty() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/Paris-20230304-duplicated.jpg");
        let path_to_format = "".to_string();
        let _placeholders = parse_placeholders(&path_to_format);
        let placeholders = map_placeholders_to_enums(&_placeholders);
        let mut path_formatter = PathFormatter::new(&path_to_format, &placeholders, None, false);
        let formatted_path = path_formatter.get_formatted_path(&path);
        let expected_path = PathBuf::from("");
        assert!(formatted_path.is_ok());
        assert!(formatted_path.unwrap() == expected_path);
    }
}
