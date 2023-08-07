use std::collections::HashMap;

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
struct Placeholder {
    full_placeholder: String,
    placeholders: Vec<String>,
}

fn parse_percent_placeholder(iter: &mut std::iter::Peekable<std::str::Chars>) -> Placeholder {
    let mut full_placeholder = "%".to_string();
    let mut current_placeholder = "%".to_string();
    let mut placeholders = Vec::new();
    println!("iter: {:?}", iter);
    iter.next();
    while let Some(next_char) = iter.next() {
        if is_word_char(next_char) {
            current_placeholder.push(next_char);
            full_placeholder.push(next_char);
            continue;
        }

        if next_char == '%' {
            placeholders.push(current_placeholder.clone());
            current_placeholder.clear();
            break;
        }
        break;
    }
    placeholders.push(current_placeholder.clone());
    Placeholder {
        full_placeholder,
        placeholders,
    }
}

fn parse_curly_placeholder(iter: &mut std::iter::Peekable<std::str::Chars>) -> Placeholder {
    let mut full_placeholder = "{".to_string();
    let mut current_placeholder = "{".to_string();
    let mut placeholders = Vec::new();

    iter.next();
    while let Some(next_char) = iter.next() {
        if next_char == '}' {
            full_placeholder.push('}');
            break;
        }
        if is_word_char(next_char) {
            current_placeholder.push(next_char);
            full_placeholder.push(next_char);
            continue;
        }
        if next_char == '%' {
            let placeholder = parse_percent_placeholder(iter);
            placeholders.extend(placeholder.placeholders);
        } else if next_char == '|' || next_char == ' ' {
            placeholders.push(current_placeholder.clone());
        }

        current_placeholder.clear();
        full_placeholder.push(next_char);
    }

    Placeholder {
        full_placeholder,
        placeholders,
    }
}

pub fn get_placeholders_map(raw_path: &str) -> HashMap<String, Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_percent_placeholder_single() {
        let input = "%placeholder";
        let mut iter = input.chars().peekable();
        let result = parse_percent_placeholder(&mut iter);

        let expected = Placeholder {
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

        let expected = Placeholder {
            full_placeholder: "%placeholder".to_string(),
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

        let expected = Placeholder {
            full_placeholder: "%placeholder".to_string(),
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

        let expected = Placeholder {
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
    fn test_parse_get_placeholders_map() {
        let input = "/home/myuser/photos/%year/{%city|%event|To sort}";
        let result = get_placeholders_map(input);

        let mut expected = HashMap::new();
        expected.insert("%placeholder".to_string(), vec!["%placeholder".to_string()]);

        assert_eq!(result, expected);
    }
    #[test]
    fn test_parse_get_placeholders_map_multiple_in_touch() {
        let input = "/home/myuser/photos/%year/{%city|%event|To sort}";
        let result = get_placeholders_map(input);

        let mut expected = HashMap::new();
        expected.insert("%placeholder".to_string(), vec!["%placeholder".to_string()]);
        expected.insert(
            "{%placeholder}".to_string(),
            vec!["%placeholder".to_string()],
        );

        assert_eq!(result, expected);
    }
    #[test]
    fn test_parse_get_placeholders_map_full_path() {
        let input = "/home/myuser/photos/%year/{%city|%event|To sort}";
        let result = get_placeholders_map(input);

        let mut expected = HashMap::new();
        expected.insert("%year".to_string(), vec!["%year".to_string()]);
        expected.insert(
            "{%city|%event|To sort}".to_string(),
            vec![
                "%city".to_string(),
                "%event".to_string(),
                "To sort".to_string(),
            ],
        );

        assert_eq!(result, expected);
    }
}
