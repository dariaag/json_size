/// Calculates the approximate size of a `serde_json::Value` in bytes.
///
/// This function estimates the memory consumption of the given `serde_json::Value` object, including its nested structures. The estimation is based on the following assumptions:
///
/// - Null, Boolean, and Number types have no additional size overhead.
/// - String sizes are based on the capacity of the internal buffer.
/// - Array sizes are calculated recursively based on the sum of each element's size.
/// - Object sizes are calculated recursively, summing the size of each key-value pair. An additional crude approximation of map entry overhead is included.
///
/// ## Parameters
/// - `v`: A reference to a `serde_json::Value` whose size will be estimated.
///
/// ## Returns
/// An estimated size of the provided JSON value in bytes.
///
/// ## Example
/// ```
/// use serde_json::json;
/// use json_size::sizeof_val;
///
/// let val = json!({
///     "name": "OpenAI",
///     "founded": 2015,
///     "services": ["chatbot", "API"]
/// });
///
/// let size = sizeof_val(&val);
/// println!("Estimated size: {} bytes", size);
/// ```
///
/// ## Caveats
/// - This estimation might not be precise for objects using arbitrary precision numbers.
/// - The estimation might vary depending on the specific architecture and implementation of the `serde_json` crate.
///
/// ## Implementation
use serde_json::Value;
use std::mem::size_of;

const STRING_OVERHEAD: usize = size_of::<String>();
const MAP_ENTRY_OVERHEAD: usize = size_of::<usize>() * 3;

pub fn sizeof_val(v: &Value) -> usize {
    size_of::<Value>()
        + match v {
            Value::Null => 0,
            Value::Bool(_) => 0,
            Value::Number(_) => 0, // incorrect if arbitrary_precision is enabled
            Value::String(s) => STRING_OVERHEAD + s.capacity(),
            Value::Array(a) => a.iter().map(sizeof_val).sum(),
            Value::Object(o) => o
                .iter()
                .map(|(k, v)| STRING_OVERHEAD + k.capacity() + sizeof_val(v) + MAP_ENTRY_OVERHEAD)
                .sum(),
        }
}

#[cfg(test)]

mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sizeof_val_null() {
        let val = json!(null);
        assert_eq!(sizeof_val(&val), std::mem::size_of::<serde_json::Value>());
    }

    #[test]
    fn test_sizeof_val_bool() {
        let val = json!(true);
        assert_eq!(sizeof_val(&val), std::mem::size_of::<serde_json::Value>());
    }

    #[test]
    fn test_sizeof_val_number() {
        let val = json!(42);
        assert_eq!(sizeof_val(&val), std::mem::size_of::<serde_json::Value>());
    }

    #[test]
    fn test_sizeof_val_string() {
        let val = json!("Hello, world!");
        let expected_size = std::mem::size_of::<serde_json::Value>()
            + String::from("Hello, world!").capacity()
            + STRING_OVERHEAD;
        assert_eq!(sizeof_val(&val), expected_size);
    }

    #[test]
    fn test_sizeof_val_array() {
        let val = json!([1, 2, 3]);
        let expected_size = std::mem::size_of::<serde_json::Value>()
            + sizeof_val(&json!(1))
            + sizeof_val(&json!(2))
            + sizeof_val(&json!(3));
        assert_eq!(sizeof_val(&val), expected_size);
    }

    #[test]
    fn test_sizeof_val_object() {
        let val = json!({"key": "value"});
        let expected_size = std::mem::size_of::<serde_json::Value>()
            + String::from("key").capacity()
            + sizeof_val(&json!("value"))
            + std::mem::size_of::<String>()
            + std::mem::size_of::<usize>() * 3;
        assert_eq!(sizeof_val(&val), expected_size);
    }

    #[test]
    fn test_sizeof_val_complex_object() {
        let val = json!({
            "name": "json_size",
            "details": {"year": 2022, "version": "v4"}
        });
        let expected_size = std::mem::size_of::<serde_json::Value>()
            + String::from("name").capacity()
            + sizeof_val(&json!("json_size"))
            + String::from("details").capacity()
            + sizeof_val(&json!({"year": 2022, "version": "v4"}))
            + std::mem::size_of::<String>() * 2
            + std::mem::size_of::<usize>() * 6; // Assuming each object entry overhead is 3 usize
        assert_eq!(sizeof_val(&val), expected_size);
    }
}
