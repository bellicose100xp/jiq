use crate::stats::types::{ElementType, ResultStats};

pub struct StatsParser;

impl StatsParser {
    pub fn parse(result: &str) -> ResultStats {
        let trimmed = result.trim();

        if trimmed.is_empty() {
            return ResultStats::Null;
        }

        if let Some(count) = Self::is_stream(trimmed) {
            return ResultStats::Stream { count };
        }

        match trimmed.chars().next() {
            Some('[') => {
                let count = Self::count_array_items(trimmed);
                let element_type = if count == 0 {
                    ElementType::Empty
                } else {
                    Self::detect_element_type(trimmed)
                };
                ResultStats::Array {
                    count,
                    element_type,
                }
            }
            Some('{') => ResultStats::Object,
            Some('"') => ResultStats::String,
            Some('t') | Some('f') => ResultStats::Boolean,
            Some('n') => ResultStats::Null,
            Some(c) if c.is_ascii_digit() || c == '-' => ResultStats::Number,
            _ => ResultStats::Null,
        }
    }

    fn count_array_items(result: &str) -> usize {
        let mut depth = 0;
        let mut comma_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut has_content = false;

        for ch in result.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' && in_string {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                if depth == 1 {
                    has_content = true;
                }
                continue;
            }

            if in_string {
                continue;
            }

            match ch {
                '[' | '{' => {
                    if depth == 1 {
                        has_content = true;
                    }
                    depth += 1;
                }
                ']' | '}' => {
                    depth -= 1;
                }
                ',' => {
                    if depth == 1 {
                        comma_count += 1;
                    }
                }
                c if !c.is_whitespace() && depth == 1 => {
                    has_content = true;
                }
                _ => {}
            }
        }

        if has_content { comma_count + 1 } else { 0 }
    }

    fn detect_element_type(result: &str) -> ElementType {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut first_type: Option<ElementType> = None;
        let mut elements_checked = 0;
        const MAX_ELEMENTS_TO_CHECK: usize = 10;

        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;

        while i < chars.len() && elements_checked < MAX_ELEMENTS_TO_CHECK {
            let ch = chars[i];

            if escape_next {
                escape_next = false;
                i += 1;
                continue;
            }

            if ch == '\\' && in_string {
                escape_next = true;
                i += 1;
                continue;
            }

            if ch == '"' {
                if depth == 1 && !in_string {
                    let element_type = ElementType::Strings;
                    match &first_type {
                        None => first_type = Some(element_type),
                        Some(t) if *t != ElementType::Strings => return ElementType::Mixed,
                        _ => {}
                    }
                    elements_checked += 1;
                }
                in_string = !in_string;
                i += 1;
                continue;
            }

            if in_string {
                i += 1;
                continue;
            }

            match ch {
                '[' => {
                    if depth == 1 {
                        let element_type = ElementType::Arrays;
                        match &first_type {
                            None => first_type = Some(element_type),
                            Some(t) if *t != ElementType::Arrays => return ElementType::Mixed,
                            _ => {}
                        }
                        elements_checked += 1;
                    }
                    depth += 1;
                }
                '{' => {
                    if depth == 1 {
                        let element_type = ElementType::Objects;
                        match &first_type {
                            None => first_type = Some(element_type),
                            Some(t) if *t != ElementType::Objects => return ElementType::Mixed,
                            _ => {}
                        }
                        elements_checked += 1;
                    }
                    depth += 1;
                }
                ']' | '}' => {
                    depth -= 1;
                }
                't' | 'f' if depth == 1 => {
                    let element_type = ElementType::Booleans;
                    match &first_type {
                        None => first_type = Some(element_type),
                        Some(t) if *t != ElementType::Booleans => return ElementType::Mixed,
                        _ => {}
                    }
                    elements_checked += 1;
                }
                'n' if depth == 1 => {
                    let element_type = ElementType::Nulls;
                    match &first_type {
                        None => first_type = Some(element_type),
                        Some(t) if *t != ElementType::Nulls => return ElementType::Mixed,
                        _ => {}
                    }
                    elements_checked += 1;
                }
                c if (c.is_ascii_digit() || c == '-') && depth == 1 => {
                    let element_type = ElementType::Numbers;
                    match &first_type {
                        None => first_type = Some(element_type),
                        Some(t) if *t != ElementType::Numbers => return ElementType::Mixed,
                        _ => {}
                    }
                    elements_checked += 1;
                }
                _ => {}
            }

            i += 1;
        }

        first_type.unwrap_or(ElementType::Empty)
    }

    fn is_stream(result: &str) -> Option<usize> {
        let mut count = 0;
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut in_value = false;

        for ch in result.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' && in_string {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                if !in_string && depth == 0 && !in_value {
                    count += 1;
                    in_value = true;
                }
                in_string = !in_string;
                continue;
            }

            if in_string {
                continue;
            }

            match ch {
                '[' | '{' => {
                    if depth == 0 && !in_value {
                        count += 1;
                        in_value = true;
                    }
                    depth += 1;
                }
                ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        in_value = false;
                    }
                }
                't' | 'f' | 'n' if depth == 0 && !in_value => {
                    count += 1;
                    in_value = true;
                }
                c if (c.is_ascii_digit() || c == '-') && depth == 0 && !in_value => {
                    count += 1;
                    in_value = true;
                }
                c if c.is_whitespace() && depth == 0 => {
                    in_value = false;
                }
                _ => {}
            }
        }

        if count > 1 { Some(count) } else { None }
    }
}

#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;
