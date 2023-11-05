use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub fn parse_json_from_iter(json_iter: Chars)
{
    parse_value(&mut json_iter.peekable());
}

pub fn parse_json_from_str(json_str: &str) -> Option<JsonValue> {
    parse_value(&mut json_str.chars().peekable())
}

fn parse_object(chars: &mut Peekable<Chars>) -> Option<JsonValue> {
    // Consume opening bracket
    match chars.next() {
        Some('{') => (),
        _ => return None,
    }

    let mut object = HashMap::new();

    loop {
        // Consume whitespaces and commas
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == ',' {
                chars.next();
            } else {
                break;
            }
        }

        // Check if the object ends
        if let Some(&c) = chars.peek() {
            if c == '}' {
                chars.next();
                break;
            }
        }

        // Parse key
        let key = match parse_string(chars) {
            Some(JsonValue::String(s)) => s,
            _ => return None,
        };

        // Consume colon
        match chars.next() {
            Some(':') => (),
            _ => return None,
        }

        // Parse value
        if let Some(value) = parse_value(chars) {
            object.insert(key, value);
        } else {
            return None;
        }

        // Consume whitespaces and commas
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == ',' {
                chars.next();
            } else {
                break;
            }
        }
    }

    Some(JsonValue::Object(object))
}

fn parse_array(chars: &mut Peekable<Chars>) -> Option<JsonValue> {

    // Consume opening bracket
    match chars.next() {
        Some('[') => (),
        _ => return None,
    }

    let mut array = Vec::new();

    loop {
        // Consume whitespaces and commas
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == ',' {
                chars.next();
            } else {
                break;
            }
        }

        // Check if the array ends
        if let Some(&c) = chars.peek() {
            if c == ']' {
                chars.next();
                break;
            }
        }

        // Parse value
        if let Some(value) = parse_value(chars) {
            array.push(value);
        } else {
            return None;
        }

        // Consume whitespaces and commas
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == ',' {
                chars.next();
            } else {
                break;
            }
        }
    }

    Some(JsonValue::Array(array))
}

fn parse_string(chars: &mut Peekable<Chars>) -> Option<JsonValue> {
    let mut s = String::new();

    // Consume opening quote
    match chars.next() {
        Some('"') => (),
        _ => return None,
    }

    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(JsonValue::String(s)),
            '\\' => {
                // Handle escape sequences
                if let Some(escaped) = chars.next() {
                    match escaped {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        '/' => s.push('/'),
                        'b' => s.push('\u{0008}'), // backspace
                        'f' => s.push('\u{000c}'), // formfeed
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'u' => {
                            // Parse Unicode escape sequence
                            let mut unicode_chars = Vec::with_capacity(4);
                            for _ in 0..4 {
                                if let Some(hex) = chars.next() {
                                    unicode_chars.push(hex);
                                } else {
                                    return None;
                                }
                            }
                            if let Ok(unicode) = u32::from_str_radix(&unicode_chars.iter().collect::<String>(), 16) {
                                if let Some(unicode_char) = std::char::from_u32(unicode) {
                                    s.push(unicode_char);
                                } else {
                                    return None;
                                }
                            } else {
                                return None;
                            }
                        }
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }
            _ => s.push(c),
        }
    }

    None
}

fn parse_number(chars: &mut Peekable<Chars>) -> Option<JsonValue> {
    let mut number_str = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || (number_str.is_empty() && (c == '-' || c == '+')) {
            number_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if let Ok(number) = number_str.parse::<f64>() {
        Some(JsonValue::Number(number))
    } else {
        None
    }
}

fn parse_bool(chars: &mut Peekable<Chars>) -> Option<JsonValue> {
    if let Some(c) = chars.peek() {
        if *c == 't' {
            // true
            let result: String = chars.take(4).collect();
            if result == "true" {
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                return Some(JsonValue::Boolean(true));
            }
        } else if *c == 'f' {
            // false
            let result: String = chars.take(5).collect();
            if result == "false" {
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                return Some(JsonValue::Boolean(false));
            }
        }
    }

    None
}

fn parse_null(chars: &mut Peekable<Chars>) -> Option<JsonValue> {
    let result: String = chars.take(4).collect();
    if result == "null" {
        chars.next();
        chars.next();
        chars.next();
        chars.next();
        Some(JsonValue::Null)
    } else {
        None
    }
}

fn parse_value(chars: &mut Peekable<Chars>) -> Option<JsonValue> {
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\n' | '\t' | '\r' => {
                chars.next();
            }
            '{' => {
                return parse_object(chars);
            }
            '}' => {
                chars.next();
                return None;
            }
            '[' => {
                return parse_array(chars);
            }
            ']' => {
                chars.next();
                return None;
            }
            '"' => {
                // chars.next();
                return parse_string(chars);
            }
            '0'..='9' | '-' => {
                return parse_number(chars);
            }
            't' | 'f' => {
                return parse_bool(chars);
            }
            'n' => {
                return parse_null(chars);
            }
            _ => {
                chars.next();
            }
        }
    }

    None
}

mod test {
    use json::parse_json_from_str as parse;
    use crate::json;

    #[test]
    fn main() {
        let json_str = r#"
        {
            "name": "John Doe",
            "age": 30,
            "is_student": false,
            "grades": [95, 87, 92],
            "address": {
                "street": "123 Main St",
                "city": "Anytown",
                "state": "CA"
            }
        }
    "#;

        let result = parse(json_str);
        assert!(matches!(result, Some(_)));
        println!("{result:?}")
    }

    #[test]
    fn raw_value() {
        let json_str = r#""raw""#;
        let result = parse(json_str);
        assert!(matches!(result, Some(json::JsonValue::String(_))));
    }
}
