use super::*;
use proptest::prelude::*;

mod char_pos_to_byte_pos {
    use super::*;

    #[test]
    fn empty_string_zero_returns_zero() {
        assert_eq!(char_pos_to_byte_pos("", 0), 0);
    }

    #[test]
    fn empty_string_past_end_returns_zero() {
        assert_eq!(char_pos_to_byte_pos("", 5), 0);
    }

    #[test]
    fn ascii_matches_char_index() {
        let s = "hello";
        assert_eq!(char_pos_to_byte_pos(s, 0), 0);
        assert_eq!(char_pos_to_byte_pos(s, 1), 1);
        assert_eq!(char_pos_to_byte_pos(s, 5), 5);
    }

    #[test]
    fn ascii_past_end_returns_byte_len() {
        assert_eq!(char_pos_to_byte_pos("hello", 10), 5);
    }

    #[test]
    fn two_byte_chars() {
        let s = "café";
        assert_eq!(char_pos_to_byte_pos(s, 0), 0);
        assert_eq!(char_pos_to_byte_pos(s, 3), 3);
        assert_eq!(char_pos_to_byte_pos(s, 4), 5);
        assert_eq!(char_pos_to_byte_pos(s, 5), 5);
    }

    #[test]
    fn three_byte_chars() {
        let s = "中文";
        assert_eq!(char_pos_to_byte_pos(s, 0), 0);
        assert_eq!(char_pos_to_byte_pos(s, 1), 3);
        assert_eq!(char_pos_to_byte_pos(s, 2), 6);
    }

    #[test]
    fn four_byte_chars() {
        let s = "👋🎉";
        assert_eq!(char_pos_to_byte_pos(s, 0), 0);
        assert_eq!(char_pos_to_byte_pos(s, 1), 4);
        assert_eq!(char_pos_to_byte_pos(s, 2), 8);
    }

    #[test]
    fn mixed_multibyte() {
        let s = "a中b👋c";
        assert_eq!(char_pos_to_byte_pos(s, 0), 0);
        assert_eq!(char_pos_to_byte_pos(s, 1), 1);
        assert_eq!(char_pos_to_byte_pos(s, 2), 4);
        assert_eq!(char_pos_to_byte_pos(s, 3), 5);
        assert_eq!(char_pos_to_byte_pos(s, 4), 9);
        assert_eq!(char_pos_to_byte_pos(s, 5), 10);
    }
}

mod byte_pos_to_char_pos {
    use super::*;

    #[test]
    fn empty_string_zero_returns_zero() {
        assert_eq!(byte_pos_to_char_pos("", 0), 0);
    }

    #[test]
    fn empty_string_past_end_returns_zero() {
        assert_eq!(byte_pos_to_char_pos("", 5), 0);
    }

    #[test]
    fn ascii_matches_byte_index() {
        let s = "hello";
        assert_eq!(byte_pos_to_char_pos(s, 0), 0);
        assert_eq!(byte_pos_to_char_pos(s, 3), 3);
        assert_eq!(byte_pos_to_char_pos(s, 5), 5);
    }

    #[test]
    fn ascii_past_end_returns_char_count() {
        assert_eq!(byte_pos_to_char_pos("hello", 100), 5);
    }

    #[test]
    fn two_byte_chars() {
        let s = "café";
        assert_eq!(byte_pos_to_char_pos(s, 0), 0);
        assert_eq!(byte_pos_to_char_pos(s, 3), 3);
        assert_eq!(byte_pos_to_char_pos(s, 5), 4);
    }

    #[test]
    fn three_byte_chars() {
        let s = "中文";
        assert_eq!(byte_pos_to_char_pos(s, 0), 0);
        assert_eq!(byte_pos_to_char_pos(s, 3), 1);
        assert_eq!(byte_pos_to_char_pos(s, 6), 2);
    }

    #[test]
    fn four_byte_chars() {
        let s = "👋🎉";
        assert_eq!(byte_pos_to_char_pos(s, 0), 0);
        assert_eq!(byte_pos_to_char_pos(s, 4), 1);
        assert_eq!(byte_pos_to_char_pos(s, 8), 2);
    }

    #[test]
    fn byte_inside_multibyte_char_rounds_to_next_boundary() {
        let s = "中";
        assert_eq!(byte_pos_to_char_pos(s, 1), 1);
        assert_eq!(byte_pos_to_char_pos(s, 2), 1);
    }

    #[test]
    fn mixed_multibyte() {
        let s = "a中b👋c";
        assert_eq!(byte_pos_to_char_pos(s, 0), 0);
        assert_eq!(byte_pos_to_char_pos(s, 1), 1);
        assert_eq!(byte_pos_to_char_pos(s, 4), 2);
        assert_eq!(byte_pos_to_char_pos(s, 5), 3);
        assert_eq!(byte_pos_to_char_pos(s, 9), 4);
        assert_eq!(byte_pos_to_char_pos(s, 10), 5);
    }
}

proptest! {
    #[test]
    fn prop_roundtrip_through_byte(s in "\\PC{0,50}", char_pos in 0usize..60) {
        let byte_pos = char_pos_to_byte_pos(&s, char_pos);
        let back = byte_pos_to_char_pos(&s, byte_pos);
        let char_count = s.chars().count();
        prop_assert_eq!(back, char_pos.min(char_count));
    }

    #[test]
    fn prop_char_pos_result_is_byte_boundary(s in "\\PC{0,50}", char_pos in 0usize..60) {
        let byte_pos = char_pos_to_byte_pos(&s, char_pos);
        prop_assert!(s.is_char_boundary(byte_pos));
    }

    #[test]
    fn prop_slice_at_result_never_panics(s in "\\PC{0,50}", char_pos in 0usize..60) {
        let byte_pos = char_pos_to_byte_pos(&s, char_pos);
        let _ = &s[..byte_pos];
        let _ = &s[byte_pos..];
    }

    #[test]
    fn prop_byte_to_char_bounded_by_char_count(s in "\\PC{0,50}", byte_pos in 0usize..200) {
        let char_pos = byte_pos_to_char_pos(&s, byte_pos);
        prop_assert!(char_pos <= s.chars().count());
    }
}
