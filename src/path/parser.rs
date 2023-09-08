use crate::placeholders::Placeholder;
use std::collections::HashMap;

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[derive(Debug, PartialEq)]
struct ParsedPlaceholder {
    full_placeholder: String,
    placeholders: Vec<String>,
}

fn parse_percent_placeholder(iter: &mut std::iter::Peekable<std::str::Chars>) -> ParsedPlaceholder {
    let mut full_placeholder = String::from("%");
    let mut current_placeholder = String::from("%");
    let mut placeholders = Vec::new();

    iter.next();
    while let Some(&next_char) = iter.peek() {
        if is_word_char(next_char) {
            current_placeholder.push(next_char);
            full_placeholder.push(next_char);
            iter.next();
        } else if next_char == '%' {
            placeholders.push(current_placeholder.clone());
            current_placeholder.clear();
            iter.next();
            break;
        } else {
            break;
        }
    }
    placeholders.push(current_placeholder.clone());
    ParsedPlaceholder {
        full_placeholder,
        placeholders,
    }
}

fn parse_curly_placeholder(iter: &mut std::iter::Peekable<std::str::Chars>) -> ParsedPlaceholder {
    let mut full_placeholder = String::from("{");
    let mut current_placeholder = String::from("{");
    let mut placeholders = Vec::new();
    let mut is_in_fallback = false;

    iter.next();
    while let Some(&next_char) = iter.peek() {
        if next_char == '}' {
            if is_in_fallback {
                placeholders.push(current_placeholder.clone());
            }
            full_placeholder.push(next_char);
            iter.next();
            break;
        }
        if next_char == '%' {
            is_in_fallback = false;
            let placeholder = parse_percent_placeholder(iter);
            full_placeholder.push_str(&placeholder.full_placeholder);
            placeholders.extend(placeholder.placeholders);
            current_placeholder.clear();
            continue;
        }
        if next_char == '|' {
            is_in_fallback = true;
            current_placeholder.clear();
            full_placeholder.push(next_char);
            iter.next();
            continue;
        } else {
            current_placeholder.push(next_char);
            full_placeholder.push(next_char);
            iter.next();
        }
    }

    ParsedPlaceholder {
        full_placeholder,
        placeholders,
    }
}

pub fn parse_placeholders(raw_path: &str) -> HashMap<String, Vec<String>> {
    let mut placeholder_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut iter = raw_path.chars().peekable();

    while let Some(&c) = iter.peek() {
        match c {
            '%' => {
                let placeholder = parse_percent_placeholder(&mut iter);
                placeholder_map
                    .entry(placeholder.full_placeholder.clone())
                    .or_insert(vec![])
                    .extend(placeholder.placeholders);
            }

            '{' => {
                let placeholder = parse_curly_placeholder(&mut iter);
                placeholder_map
                    .entry(placeholder.full_placeholder.clone())
                    .or_insert(vec![])
                    .extend(placeholder.placeholders);
            }

            _ => {
                iter.next(); // Skip the character if it is not '%' or '{'
            }
        }
    }

    placeholder_map
}

/// Maps placeholders to enums.
///
/// # Arguments
///
/// * `_placeholders` - A reference to a HashMap containing placeholder strings.
///
/// # Returns
///
/// A HashMap containing the mappings from placeholder strings to Placeholder enums.
pub fn map_placeholders_to_enums(
    _placeholders: &HashMap<String, Vec<String>>,
) -> HashMap<String, HashMap<String, Placeholder>> {
    let mut placeholders: HashMap<String, HashMap<String, Placeholder>> = HashMap::new();

    for (full, placeholders_string) in _placeholders {
        let mut placeholders_map_string_enum: HashMap<String, Placeholder> = HashMap::new();

        for placeholder_string in placeholders_string {
            let placeholder_enum = Placeholder::from_string(placeholder_string);
            placeholders_map_string_enum.insert(placeholder_string.to_string(), placeholder_enum);
        }
        placeholders.insert(full.clone(), placeholders_map_string_enum);
    }
    placeholders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_percent_placeholder_single() {
        let input = "%placeholder";
        let mut iter = input.chars().peekable();
        let result = parse_percent_placeholder(&mut iter);

        let expected = ParsedPlaceholder {
            full_placeholder: "%placeholder".to_string(),
            placeholders: vec!["%placeholder".to_string()],
        };

        assert_eq!(result.placeholders, expected.placeholders);
        assert_eq!(result.full_placeholder, expected.full_placeholder);
    }
    #[test]
    fn test_parse_curly_placeholder_single() {
        let input = "{%placeholder}";
        let mut iter = input.chars().peekable();
        let result = parse_curly_placeholder(&mut iter);

        let expected = ParsedPlaceholder {
            full_placeholder: "{%placeholder}".to_string(),
            placeholders: vec!["%placeholder".to_string()],
        };

        assert_eq!(result.placeholders, expected.placeholders);
        assert_eq!(result.full_placeholder, expected.full_placeholder);
    }

    #[test]
    fn test_parse_curly_placeholder_multiple() {
        let input = "{%placeholder|%second_placeholder}";
        let mut iter = input.chars().peekable();
        let result = parse_curly_placeholder(&mut iter);

        let expected = ParsedPlaceholder {
            full_placeholder: "{%placeholder|%second_placeholder}".to_string(),
            placeholders: vec![
                "%placeholder".to_string(),
                "%second_placeholder".to_string(),
            ],
        };

        assert_eq!(result.placeholders, expected.placeholders);
        assert_eq!(result.full_placeholder, expected.full_placeholder);
    }
    #[test]
    fn test_parse_curly_placeholder_multiple_fallback() {
        let input = "{%placeholder|%second_placeholder|fallback}";
        let mut iter = input.chars().peekable();
        let result = parse_curly_placeholder(&mut iter);

        let expected = ParsedPlaceholder {
            full_placeholder: "{%placeholder|%second_placeholder|fallback}".to_string(),
            placeholders: vec![
                "%placeholder".to_string(),
                "%second_placeholder".to_string(),
                "fallback".to_string(),
            ],
        };

        assert_eq!(result.placeholders, expected.placeholders);
        assert_eq!(result.full_placeholder, expected.full_placeholder);
    }
    #[test]
    fn test_parse_curly_placeholder_multiple_last_fallback_empty() {
        let input = "{%placeholder|%second_placeholder|}";
        let mut iter = input.chars().peekable();
        let result = parse_curly_placeholder(&mut iter);

        let expected = ParsedPlaceholder {
            full_placeholder: "{%placeholder|%second_placeholder|}".to_string(),
            placeholders: vec![
                "%placeholder".to_string(),
                "%second_placeholder".to_string(),
                "".to_string(),
            ],
        };

        assert_eq!(result.placeholders, expected.placeholders);
        assert_eq!(result.full_placeholder, expected.full_placeholder);
    }
    #[test]
    fn test_parse_parse_placeholders_full_path() {
        let input = "/home/myuser/photos/%year/{%city|%camera_brand|To sort}";
        let result = parse_placeholders(input);

        let mut expected = HashMap::new();
        expected.insert("%year".to_string(), vec!["%year".to_string()]);
        expected.insert(
            "{%city|%camera_brand|To sort}".to_string(),
            vec![
                "%city".to_string(),
                "%camera_brand".to_string(),
                "To sort".to_string(),
            ],
        );

        assert_eq!(result, expected);
    }
}
