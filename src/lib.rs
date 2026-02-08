//! # json_size
//!
//! Estimate the in-memory heap size of [`serde_json::Value`] trees.
//!
//! This is useful when you need to enforce memory budgets on deserialized JSON
//! — for instance, capping the size of user-submitted payloads in a web service,
//! or deciding when to evict entries from an in-memory cache.
//!
//! # Accuracy
//!
//! The estimates account for heap allocations made by `String`, `Vec`, and
//! `Map<String, Value>`, but do not capture allocator overhead, alignment
//! padding, or the internal representation of `Number` when the
//! `arbitrary_precision` feature of `serde_json` is enabled.
//!
//! # Example
//!
//! ```
//! use serde_json::json;
//! use json_size::sizeof_val;
//!
//! let v = json!({"key": [1, 2, 3], "name": "hello"});
//! let bytes = sizeof_val(&v);
//! assert!(bytes > 0);
//! ```

use serde_json::Value;
use std::mem::size_of;

const STRING_OVERHEAD: usize = size_of::<String>();
const MAP_ENTRY_OVERHEAD: usize = size_of::<usize>() * 3;
/// Returns an estimate of the total heap memory consumed by `v`, in bytes.
///
/// The estimate includes the stack-size of the root `Value` plus the heap
/// allocations reachable from it. For objects, a crude per-entry overhead
/// of `3 * size_of::<usize>()` is added to approximate the `BTreeMap`
/// node overhead.
///
/// # Caveats
///
/// - `Number` is treated as zero additional heap allocation, which is
///   incorrect when `serde_json`'s `arbitrary_precision` feature stores
///   numbers as heap-allocated strings.
/// - `Array` capacity beyond its length is not counted (only the elements
///   that exist are traversed).
pub fn sizeof_val(v: &Value) -> usize {
    size_of::<Value>()
        + match v {
            Value::Null => 0,
            Value::Bool(_) => 0,
            Value::Number(_n) => {
                #[cfg(feature = "arbitrary_precision")]
                {
                    16
                }
                #[cfg(not(feature = "arbitrary_precision"))]
                {
                    0
                }
            }
            Value::String(s) => s.capacity(),
            Value::Array(a) => a.iter().map(sizeof_val).sum(),
            Value::Object(o) => o
                .iter()
                .map(|(k, v)| STRING_OVERHEAD + k.capacity() + sizeof_val(v) + MAP_ENTRY_OVERHEAD)
                .sum(),
        }
}
/// Returns the estimated heap size of `v`, excluding the stack size of the
/// root `Value` itself.
///
/// Use this when you already have the `Value` on the stack and want to know
/// only the heap cost. Equivalent to `sizeof_val(v) - size_of::<Value>()`.
pub fn heap_size(v: &Value) -> usize {
    sizeof_val(v) - size_of::<Value>()
}

/// Returns `true` if the estimated size of `v` exceeds `limit` bytes.
///
/// This short-circuits: it stops traversing the tree as soon as the
/// running total exceeds `limit`, which can be significantly faster
/// than computing the full size of a deeply nested value.
pub fn exceeds_size(v: &Value, limit: usize) -> bool {
    exceeds_size_inner(v, limit, &mut 0)
}

fn exceeds_size_inner(v: &Value, limit: usize, acc: &mut usize) -> bool {
    *acc += size_of::<Value>();
    if *acc > limit {
        return true;
    }
    match v {
        Value::Null | Value::Bool(_) => false,
        Value::Number(_) => {
            #[cfg(feature = "arbitrary_precision")]
            {
                *acc += 16;
            }
            *acc > limit
        }
        Value::String(s) => {
            *acc += s.capacity();
            *acc > limit
        }
        Value::Array(a) => a.iter().any(|elem| exceeds_size_inner(elem, limit, acc)),
        Value::Object(o) => o.iter().any(|(k, v)| {
            *acc += size_of::<String>() + k.capacity() + size_of::<usize>() * 3;
            if *acc > limit {
                return true;
            }
            exceeds_size_inner(v, limit, acc)
        }),
    }
}

/// Detailed size breakdown of a [`Value`] tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeBreakdown {
    /// Total estimated bytes (same as [`sizeof_val`]).
    pub total: usize,
    /// Bytes attributed to string content (keys and string values).
    pub strings: usize,
    /// Number of `Value` nodes in the tree.
    pub node_count: usize,
    /// Maximum nesting depth encountered.
    pub max_depth: usize,
}

/// Computes a [`SizeBreakdown`] of the value tree rooted at `v`.
///
/// This traverses the entire tree once, collecting size and structural
/// metadata that can be useful for diagnostics and logging.
pub fn size_breakdown(v: &Value) -> SizeBreakdown {
    let mut breakdown = SizeBreakdown {
        total: 0,
        strings: 0,
        node_count: 0,
        max_depth: 0,
    };
    size_breakdown_inner(v, 0, &mut breakdown);
    breakdown
}

fn size_breakdown_inner(v: &Value, depth: usize, bd: &mut SizeBreakdown) {
    bd.node_count += 1;
    bd.max_depth = bd.max_depth.max(depth);
    bd.total += size_of::<Value>();

    match v {
        Value::Null | Value::Bool(_) => {}
        Value::Number(_) => {
            #[cfg(feature = "arbitrary_precision")]
            {
                bd.total += 16;
            }
        }
        Value::String(s) => {
            bd.total += s.capacity();
            bd.strings += s.capacity();
        }
        Value::Array(a) => {
            for elem in a {
                size_breakdown_inner(elem, depth + 1, bd);
            }
        }
        Value::Object(o) => {
            for (k, v) in o {
                let key_cost = size_of::<String>() + k.capacity() + size_of::<usize>() * 3;
                bd.total += key_cost;
                bd.strings += k.capacity();
                size_breakdown_inner(v, depth + 1, bd);
            }
        }
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
        #[cfg(feature = "arbitrary_precision")]
        let expected = std::mem::size_of::<serde_json::Value>() + 16;
        #[cfg(not(feature = "arbitrary_precision"))]
        let expected = std::mem::size_of::<serde_json::Value>();
        assert_eq!(sizeof_val(&val), expected);
    }

    #[test]

    fn test_sizeof_val_string() {
        let val = json!("Hello, world!");
        let expected_size =
            std::mem::size_of::<serde_json::Value>() + String::from("Hello, world!").capacity();
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

    #[test]
    fn null_is_just_the_enum() {
        assert_eq!(sizeof_val(&Value::Null), size_of::<Value>());
    }

    #[test]
    fn bool_is_just_the_enum() {
        assert_eq!(sizeof_val(&json!(true)), size_of::<Value>());
        assert_eq!(sizeof_val(&json!(false)), size_of::<Value>());
    }

    #[test]
    fn string_accounts_for_capacity() {
        let v = json!("hello");
        let size = sizeof_val(&v);
        // Must be at least the enum + the 5 bytes of content
        assert!(size >= size_of::<Value>() + 5);
    }

    #[test]
    fn empty_array_is_just_the_enum() {
        let v = json!([]);
        assert_eq!(sizeof_val(&v), size_of::<Value>());
    }

    #[test]
    fn array_sums_children() {
        let v = json!(["a", "b"]);
        let size = sizeof_val(&v);
        // At minimum: root enum + 2 child enums + string content
        assert!(size >= size_of::<Value>() * 3 + 2);
    }

    #[test]
    fn nested_objects_accumulate() {
        let shallow = json!({"a": 1});
        let deep = json!({"a": {"b": {"c": 1}}});
        assert!(sizeof_val(&deep) > sizeof_val(&shallow));
    }

    #[test]
    fn empty_object_is_just_the_enum() {
        let v = json!({});
        assert_eq!(sizeof_val(&v), size_of::<Value>());
    }

    #[test]
    fn exceeds_size_short_circuits() {
        // Build a value that's definitely > 100 bytes
        let v = json!({"a": "x".repeat(200)});
        assert!(exceeds_size(&v, 100));
        assert!(!exceeds_size(&v, 1_000_000));
    }

    #[test]
    fn exceeds_size_zero_limit() {
        // Any non-trivial value exceeds a zero-byte limit
        assert!(exceeds_size(&json!(null), 0));
    }

    #[test]
    fn breakdown_counts_nodes() {
        let v = json!({"a": [1, 2, 3], "b": "hello"});
        let bd = size_breakdown(&v);
        // root object + "a" array + 3 numbers + "b" string = 6 nodes
        assert_eq!(bd.node_count, 6);
    }

    #[test]
    fn breakdown_tracks_depth() {
        let v = json!({"a": {"b": {"c": 1}}});
        let bd = size_breakdown(&v);
        assert_eq!(bd.max_depth, 3); // root=0, a=1, b=2, c=3
    }

    #[test]
    fn breakdown_total_matches_sizeof_val() {
        let v = json!({"key": [1, "two", null, true, {"nested": "value"}]});
        let bd = size_breakdown(&v);
        assert_eq!(bd.total, sizeof_val(&v));
    }

    #[test]
    fn large_string_dominates_size() {
        let big = "x".repeat(10_000);
        let v = json!({"payload": big});
        let bd = size_breakdown(&v);
        assert!(bd.strings >= 10_000);
        assert!(bd.strings as f64 / bd.total as f64 > 0.8);
    }
}
