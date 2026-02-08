```markdown
# json_size

Estimate the in-memory heap size of `serde_json::Value` trees.

[![CI](https://github.com/dariaag/json_size/actions/workflows/ci.yml/badge.svg)](https://github.com/dariaag/json_size/actions)
[![crates.io](https://img.shields.io/crates/v/json_size.svg)](https://crates.io/crates/json_size)
[![docs.rs](https://docs.rs/json_size/badge.svg)](https://docs.rs/json_size)

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

## API

| Function | Description |
|---|---|
| `sizeof_val(&Value) -> usize` | Total estimated bytes including the root `Value` |
| `heap_size(&Value) -> usize` | Heap cost only, excluding the root `Value`'s stack size |
| `exceeds_size(&Value, usize) -> bool` | Short-circuiting check against a byte budget |
| `size_breakdown(&Value) -> SizeBreakdown` | Single-pass collection of total size, string bytes, node count, and max depth |

## Accuracy

Estimates account for heap allocations by `String`, `Vec`, and `BTreeMap` entries, including per-entry node overhead. They do not capture allocator overhead, alignment padding, or `Vec`/`String` excess capacity beyond content length.

When the `arbitrary_precision` feature flag is enabled, `Number` heap cost is approximated at 16 bytes. Without it, `Number` is treated as zero additional heap allocation (the value lives inline in the `Value` enum).

## Feature flags

| Flag | Default | Effect |
|---|---|---|
| `arbitrary_precision` | off | Estimates heap cost of `Number` values stored as strings |

## Benchmarks

Run with `cargo bench`. Representative results on Apple M1:

| Benchmark | Time |
|---|---|
| `sizeof_val` small object | ~5 ns |
| `sizeof_val` 100-level deep nesting | ~500 ns |
| `sizeof_val` 1000-key flat object | ~1.8 µs |
| `exceeds_size` early exit (1000 keys, low limit) | ~4 ns |
| `size_breakdown` 1000-key flat object | ~2.6 µs |

## License

MIT
```