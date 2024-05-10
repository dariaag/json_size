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

pub fn sizeof_val(v: &serde_json::Value) -> usize {
    std::mem::size_of::<serde_json::Value>()
        + match v {
            serde_json::Value::Null => 0,
            serde_json::Value::Bool(_) => 0,
            serde_json::Value::Number(_) => 0, // incorrect if arbitrary_precision is enabled
            serde_json::Value::String(s) => s.capacity(),
            serde_json::Value::Array(a) => a.iter().map(sizeof_val).sum(),
            serde_json::Value::Object(o) => o
                .iter()
                .map(|(k, v)| {
                    std::mem::size_of::<String>()
                        + k.capacity()
                        + sizeof_val(v)
                        + std::mem::size_of::<usize>() * 3 //crude approximation, each map entry has 3 words of overhead
                })
                .sum(),
        }
}
