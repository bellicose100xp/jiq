/// JSON tree navigator for autocomplete path resolution.
///
/// Navigates pre-parsed JSON structures following path segments
/// to find nested values for field suggestions.
use serde_json::Value;

use super::path_parser::PathSegment;

/// Maximum number of array elements to sample when collecting field names.
/// Unions keys from the first N elements to surface fields that only
/// appear in non-first elements of heterogeneous arrays.
pub const ARRAY_SAMPLE_SIZE: usize = 10;

/// Navigate a JSON tree following path segments.
///
/// Returns a borrowed reference to the nested value, or None if the path
/// doesn't exist or encounters a type mismatch.
///
/// # Array Handling
/// - `ArrayIterator` (`.[]`) uses the first element (industry standard for autocompletion)
/// - `ArrayIndex` (`.[n]`) accesses the specific index
/// - Negative indices are converted to usize (will likely fail for large negative values)
///
/// # Examples
/// ```
/// // Given: {"user": {"name": "Alice"}}
/// // navigate(json, &[Field("user"), Field("name")]) → Some("Alice")
///
/// // Given: [{"id": 1}, {"id": 2}]
/// // navigate(json, &[ArrayIterator, Field("id")]) → Some(1)
/// ```
pub fn navigate<'a>(root: &'a Value, segments: &[PathSegment]) -> Option<&'a Value> {
    let mut current = root;

    for segment in segments {
        current = match segment {
            PathSegment::Field(name) | PathSegment::OptionalField(name) => match current {
                Value::Object(map) => map.get(name)?,
                _ => return None,
            },
            PathSegment::ArrayIterator => match current {
                Value::Array(arr) => arr.first()?,
                _ => return None,
            },
            PathSegment::ArrayIndex(i) => {
                match current {
                    Value::Array(arr) => {
                        let index = if *i < 0 {
                            // Negative index: count from end
                            let len = arr.len() as i64;
                            let adjusted = len + i;
                            if adjusted < 0 {
                                return None;
                            }
                            adjusted as usize
                        } else {
                            *i as usize
                        };
                        arr.get(index)?
                    }
                    _ => return None,
                }
            }
        };
    }

    Some(current)
}
