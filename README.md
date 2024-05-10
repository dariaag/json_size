# sizeof_val - Estimate JSON Value Size

`sizeof_val` is a Rust function that calculates an approximate size of a `serde_json::Value` in bytes. It estimates the memory consumption of various types of JSON data and their nested structures.

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
   pub fn sizeof_val(v: &serde_json::Value) -> usize {
       std::mem::size_of::<serde_json::Value>()
           + match v {
               serde_json::Value::Null => 0,
               serde_json::Value::Bool(_) => 0,
               serde_json::Value::Number(_) => 0,
               serde_json::Value::String(s) => s.capacity(),
               serde_json::Value::Array(a) => a.iter().map(sizeof_val).sum(),
               serde_json::Value::Object(o) => o
                   .iter()
                   .map(|(k, v)| {
                       std::mem::size_of::<String>()
                           + k.capacity()
                           + sizeof_val(v)
                           + std::mem::size_of::<usize>() * 3 // crude approximation of overhead
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
        "name": "OpenAI",
        "founded": 2015,
        "services": ["chatbot", "API"]
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
