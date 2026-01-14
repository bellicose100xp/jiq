use super::*;
use tui_textarea::TextArea;

fn textarea_with_text(text: &str) -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.insert_str(text);
    textarea.move_cursor(CursorMove::Head);
    textarea
}

fn move_cursor_to(textarea: &mut TextArea, pos: usize) {
    textarea.move_cursor(CursorMove::Head);
    for _ in 0..pos {
        textarea.move_cursor(CursorMove::Forward);
    }
}

mod search_direction_tests {
    use super::*;

    #[test]
    fn opposite_of_forward_is_backward() {
        assert_eq!(
            SearchDirection::Forward.opposite(),
            SearchDirection::Backward
        );
    }

    #[test]
    fn opposite_of_backward_is_forward() {
        assert_eq!(
            SearchDirection::Backward.opposite(),
            SearchDirection::Forward
        );
    }
}

mod find_char_position_tests {
    use super::*;

    #[test]
    fn find_forward_basic() {
        let text = ".name.first";
        let result = find_char_position(text, 0, '.', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, Some(5));
    }

    #[test]
    fn find_forward_first_match() {
        let text = "abcabc";
        let result = find_char_position(text, 0, 'b', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn find_forward_skips_current_position() {
        let text = "abcabc";
        let result = find_char_position(text, 1, 'b', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, Some(4));
    }

    #[test]
    fn find_forward_not_found() {
        let text = "abcdef";
        let result = find_char_position(text, 0, 'z', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, None);
    }

    #[test]
    fn find_forward_at_end_returns_none() {
        let text = "abc";
        let result = find_char_position(text, 2, 'x', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, None);
    }

    #[test]
    fn find_forward_past_end_returns_none() {
        let text = "abc";
        let result = find_char_position(text, 3, 'a', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, None);
    }

    #[test]
    fn find_backward_basic() {
        let text = ".name.first";
        let result = find_char_position(text, 10, '.', SearchDirection::Backward, SearchType::Find);
        assert_eq!(result, Some(5));
    }

    #[test]
    fn find_backward_first_match() {
        let text = "abcabc";
        let result = find_char_position(text, 5, 'b', SearchDirection::Backward, SearchType::Find);
        assert_eq!(result, Some(4));
    }

    #[test]
    fn find_backward_skips_current_position() {
        let text = "abcabc";
        let result = find_char_position(text, 4, 'b', SearchDirection::Backward, SearchType::Find);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn find_backward_not_found() {
        let text = "abcdef";
        let result = find_char_position(text, 5, 'z', SearchDirection::Backward, SearchType::Find);
        assert_eq!(result, None);
    }

    #[test]
    fn find_backward_at_start_returns_none() {
        let text = "abc";
        let result = find_char_position(text, 0, 'a', SearchDirection::Backward, SearchType::Find);
        assert_eq!(result, None);
    }

    #[test]
    fn till_forward_basic() {
        let text = "abcdef";
        let result = find_char_position(text, 0, 'd', SearchDirection::Forward, SearchType::Till);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn till_forward_adjacent_char() {
        let text = "abcdef";
        let result = find_char_position(text, 0, 'b', SearchDirection::Forward, SearchType::Till);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn till_forward_not_found() {
        let text = "abcdef";
        let result = find_char_position(text, 0, 'z', SearchDirection::Forward, SearchType::Till);
        assert_eq!(result, None);
    }

    #[test]
    fn till_backward_basic() {
        let text = "abcdef";
        let result = find_char_position(text, 5, 'b', SearchDirection::Backward, SearchType::Till);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn till_backward_adjacent_char() {
        let text = "abcdef";
        let result = find_char_position(text, 2, 'a', SearchDirection::Backward, SearchType::Till);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn till_backward_not_found() {
        let text = "abcdef";
        let result = find_char_position(text, 5, 'z', SearchDirection::Backward, SearchType::Till);
        assert_eq!(result, None);
    }

    #[test]
    fn find_forward_multiple_dots() {
        let text = "a.b.c.d";
        let result = find_char_position(text, 0, '.', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, Some(1));

        let result = find_char_position(text, 2, '.', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn find_unicode_chars() {
        let text = "hello world";
        let result = find_char_position(text, 0, 'w', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, Some(6));
    }

    #[test]
    fn empty_string_returns_none() {
        let text = "";
        let result = find_char_position(text, 0, 'a', SearchDirection::Forward, SearchType::Find);
        assert_eq!(result, None);
    }
}

mod execute_char_search_tests {
    use super::*;

    #[test]
    fn execute_find_forward_moves_cursor() {
        let mut textarea = textarea_with_text(".name.first");
        let found = execute_char_search(
            &mut textarea,
            '.',
            SearchDirection::Forward,
            SearchType::Find,
        );
        assert!(found);
        assert_eq!(textarea.cursor().1, 5);
    }

    #[test]
    fn execute_find_forward_not_found_keeps_cursor() {
        let mut textarea = textarea_with_text("abcdef");
        let found = execute_char_search(
            &mut textarea,
            'z',
            SearchDirection::Forward,
            SearchType::Find,
        );
        assert!(!found);
        assert_eq!(textarea.cursor().1, 0);
    }

    #[test]
    fn execute_find_backward_moves_cursor() {
        let mut textarea = textarea_with_text(".name.first");
        move_cursor_to(&mut textarea, 10);
        let found = execute_char_search(
            &mut textarea,
            '.',
            SearchDirection::Backward,
            SearchType::Find,
        );
        assert!(found);
        assert_eq!(textarea.cursor().1, 5);
    }

    #[test]
    fn execute_find_backward_not_found_keeps_cursor() {
        let mut textarea = textarea_with_text("abcdef");
        move_cursor_to(&mut textarea, 5);
        let found = execute_char_search(
            &mut textarea,
            'z',
            SearchDirection::Backward,
            SearchType::Find,
        );
        assert!(!found);
        assert_eq!(textarea.cursor().1, 5);
    }

    #[test]
    fn execute_till_forward_moves_before_char() {
        let mut textarea = textarea_with_text("abcdef");
        let found = execute_char_search(
            &mut textarea,
            'd',
            SearchDirection::Forward,
            SearchType::Till,
        );
        assert!(found);
        assert_eq!(textarea.cursor().1, 2);
    }

    #[test]
    fn execute_till_backward_moves_after_char() {
        let mut textarea = textarea_with_text("abcdef");
        move_cursor_to(&mut textarea, 5);
        let found = execute_char_search(
            &mut textarea,
            'b',
            SearchDirection::Backward,
            SearchType::Till,
        );
        assert!(found);
        assert_eq!(textarea.cursor().1, 2);
    }

    #[test]
    fn execute_from_middle_position() {
        let mut textarea = textarea_with_text(".name.first.last");
        move_cursor_to(&mut textarea, 6);
        let found = execute_char_search(
            &mut textarea,
            '.',
            SearchDirection::Forward,
            SearchType::Find,
        );
        assert!(found);
        assert_eq!(textarea.cursor().1, 11);
    }

    #[test]
    fn execute_empty_textarea_returns_false() {
        let mut textarea = TextArea::default();
        let found = execute_char_search(
            &mut textarea,
            'a',
            SearchDirection::Forward,
            SearchType::Find,
        );
        assert!(!found);
    }
}

mod char_search_state_tests {
    use super::*;

    #[test]
    fn state_stores_all_fields() {
        let state = CharSearchState {
            character: 'x',
            direction: SearchDirection::Forward,
            search_type: SearchType::Find,
        };
        assert_eq!(state.character, 'x');
        assert_eq!(state.direction, SearchDirection::Forward);
        assert_eq!(state.search_type, SearchType::Find);
    }

    #[test]
    fn state_equality() {
        let state1 = CharSearchState {
            character: 'a',
            direction: SearchDirection::Forward,
            search_type: SearchType::Find,
        };
        let state2 = CharSearchState {
            character: 'a',
            direction: SearchDirection::Forward,
            search_type: SearchType::Find,
        };
        assert_eq!(state1, state2);
    }

    #[test]
    fn state_inequality_different_char() {
        let state1 = CharSearchState {
            character: 'a',
            direction: SearchDirection::Forward,
            search_type: SearchType::Find,
        };
        let state2 = CharSearchState {
            character: 'b',
            direction: SearchDirection::Forward,
            search_type: SearchType::Find,
        };
        assert_ne!(state1, state2);
    }

    #[test]
    fn state_inequality_different_direction() {
        let state1 = CharSearchState {
            character: 'a',
            direction: SearchDirection::Forward,
            search_type: SearchType::Find,
        };
        let state2 = CharSearchState {
            character: 'a',
            direction: SearchDirection::Backward,
            search_type: SearchType::Find,
        };
        assert_ne!(state1, state2);
    }

    #[test]
    fn state_inequality_different_type() {
        let state1 = CharSearchState {
            character: 'a',
            direction: SearchDirection::Forward,
            search_type: SearchType::Find,
        };
        let state2 = CharSearchState {
            character: 'a',
            direction: SearchDirection::Forward,
            search_type: SearchType::Till,
        };
        assert_ne!(state1, state2);
    }
}
