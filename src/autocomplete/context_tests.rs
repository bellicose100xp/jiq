//! Tests for autocomplete context analysis

#[path = "context_tests/common.rs"]
mod common;

#[path = "context_tests/basic_context_tests.rs"]
mod basic_context_tests;

#[path = "context_tests/char_before_field_tests.rs"]
mod char_before_field_tests;

#[path = "context_tests/object_key_context_tests.rs"]
mod object_key_context_tests;

#[path = "context_tests/property_tests.rs"]
mod property_tests;

#[path = "context_tests/element_context_tests.rs"]
mod element_context_tests;
