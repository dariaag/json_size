# sizeof_val - Estimate JSON Value Size

`sizeof_val` is a Rust function that calculates an approximate size of a `serde_json::Value` in bytes. It estimates the memory consumption of various types of JSON data and their nested structures.
[Original code](https://stackoverflow.com/questions/76454260/rust-serde-get-runtime-heap-size-of-vecserde-jsonvalue)

## Usage

### Adding Dependencies

Add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
serde = "1.0"
serde_json = "1.0"
```

### Importing and Using the Function

To use the `sizeof_val` function, follow these steps:

1. **Import the necessary modules:**

   ```rust
   use serde_json::{Value, json};
   ```

2. **Define the `sizeof_val` function:**

   ```rust
   use serde_json::Value;
   use std::mem::size_of;
   pub fn sizeof_val(v: &serde_json::Value) -> usize {
    size_of::<serde_json::Value>()
        + match v {
            Value::Null => 0,
            Value::Bool(_) => 0,
            Value::Number(_) => 0, // incorrect if arbitrary_precision is enabled
            Value::String(s) => s.capacity(),
            Value::Array(a) => a.iter().map(sizeof_val).sum(),
            Value::Object(o) => o
                .iter()
                .map(|(k, v)| {
                    size_of::<String>() + k.capacity() + sizeof_val(v) + size_of::<usize>() * 3
                    //crude approximation, each map entry has 3 words of overhead
                })
                .sum(),
        }
   }
   ```

3. **Use the function to estimate the size of a JSON value:**

   ```rust
   fn main() {
       let val = json!({
           "name": "OpenAI",
           "founded": 2015,
           "services": ["chatbot", "API"]
       });

       let size = sizeof_val(&val);
       println!("Estimated size: {} bytes", size);
   }
   ```

### Example

The following example demonstrates the use of the `sizeof_val` function:

```rust
use serde_json::{Value, json};

fn main() {
    let val = json!({
        "name": "bread",
        "amount": 2,

    });

    let size = sizeof_val(&val);
    println!("Estimated size: {} bytes", size);
}
```

### Caveats

- The estimation might not be precise for objects using arbitrary precision numbers.
- The estimation might vary depending on the specific architecture and implementation of the `serde_json` crate.

## Contributing

Feel free to submit pull requests or open issues for any improvements or bugs related to the `sizeof_val` function.

## License

This project is licensed under the [MIT License](LICENSE).
# json_size

Estimate the in-memory heap size of `serde_json::Value` trees.

[![CI](https://github.com/dariaag/json_size/actions/workflows/ci.yml/badge.svg)](...) 
[![crates.io](https://img.shields.io/crates/v/json_size.svg)](...) 
[![docs.rs](https://docs.rs/json_size/badge.svg)](...) 

## When to use this

- Enforcing payload size limits in web services before processing
- Cache eviction decisions based on memory pressure  
- Logging and diagnostics for JSON-heavy pipelines

## Quick start
```toml
[dependencies]
json_size = "0.2"
```
```rust
use json_size::{sizeof_val, exceeds_size, size_breakdown};
use serde_json::json;

let v = json!({"users": [{"name": "alice"}, {"name": "bob"}]});

// Total estimated heap size
let bytes = sizeof_val(&v);

// Fast check against a budget (short-circuits on large trees)
if exceeds_size(&v, 1_048_576) {
    eprintln!("payload exceeds 1MB limit");
}

// Diagnostic breakdown
let bd = size_breakdown(&v);
println!("{} nodes, max depth {}, {}B in strings", 
    bd.node_count, bd.max_depth, bd.strings);
```

## Accuracy

[honest description of what it does and doesn't capture]

## License

MIT