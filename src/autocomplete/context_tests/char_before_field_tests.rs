use crate::autocomplete::find_char_before_field_access;

#[test]
fn test_char_before_field_after_pipe() {
    assert_eq!(
        find_char_before_field_access(".services | .", ""),
        Some('|')
    );
    assert_eq!(
        find_char_before_field_access(".services | .ser", "ser"),
        Some('|')
    );
}

#[test]
fn test_char_before_field_after_dot() {
    assert_eq!(find_char_before_field_access(".services.", ""), Some('s'));
    assert_eq!(
        find_char_before_field_access(".services.na", "na"),
        Some('s')
    );
}

#[test]
fn test_char_before_field_after_brackets() {
    assert_eq!(find_char_before_field_access(".services[].", ""), Some(']'));
    assert_eq!(
        find_char_before_field_access(".services[0].", ""),
        Some(']')
    );
}

#[test]
fn test_char_before_field_after_question() {
    assert_eq!(find_char_before_field_access(".services?.", ""), Some('?'));
    assert_eq!(
        find_char_before_field_access(".services?.na", "na"),
        Some('?')
    );
}

#[test]
fn test_char_before_field_in_constructors() {
    assert_eq!(find_char_before_field_access("[.", ""), Some('['));
    assert_eq!(find_char_before_field_access("[.a, .", ""), Some(','));
    assert_eq!(find_char_before_field_access("{name: .", ""), Some(':'));
    assert_eq!(find_char_before_field_access("{.", ""), Some('{'));
}

#[test]
fn test_char_before_field_in_functions() {
    assert_eq!(find_char_before_field_access("map(.", ""), Some('('));
    assert_eq!(
        find_char_before_field_access("select(.active).", ""),
        Some(')')
    );
}

#[test]
fn test_char_before_field_with_semicolon() {
    assert_eq!(find_char_before_field_access(".a; .", ""), Some(';'));
}

#[test]
fn test_char_before_field_at_start() {
    assert_eq!(find_char_before_field_access(".", ""), None);
    assert_eq!(find_char_before_field_access(".na", "na"), None);
}
