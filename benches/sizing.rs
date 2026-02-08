use criterion::{black_box, criterion_group, criterion_main, Criterion};
use json_size::{sizeof_val, exceeds_size, size_breakdown};
use serde_json::json;

fn build_deep_json(depth: usize) -> serde_json::Value {
    let mut v = json!(42);
    for i in 0..depth {
        v = json!({ format!("level_{}", i): v });
    }
    v
}
fn bench_breakdown(c: &mut Criterion) {
    let small = json!({"name": "test", "value": 42});
    let deep = build_deep_json(100);
    let wide = build_wide_json(1000);

    c.bench_function("size_breakdown small", |b| {
        b.iter(|| size_breakdown(black_box(&small)))
    });
    c.bench_function("size_breakdown deep_100", |b| {
        b.iter(|| size_breakdown(black_box(&deep)))
    });
    c.bench_function("size_breakdown wide_1000", |b| {
        b.iter(|| size_breakdown(black_box(&wide)))
    });
}

fn build_wide_json(width: usize) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for i in 0..width {
        map.insert(format!("key_{}", i), json!("value"));
    }
    serde_json::Value::Object(map)
}

fn bench_sizeof(c: &mut Criterion) {
    let small = json!({"name": "test", "value": 42});
    let deep = build_deep_json(100);
    let wide = build_wide_json(1000);

    c.bench_function("sizeof_val small", |b| {
        b.iter(|| sizeof_val(black_box(&small)))
    });
    c.bench_function("sizeof_val deep_100", |b| {
        b.iter(|| sizeof_val(black_box(&deep)))
    });
    c.bench_function("sizeof_val wide_1000", |b| {
        b.iter(|| sizeof_val(black_box(&wide)))
    });
}

fn bench_exceeds(c: &mut Criterion) {
    let wide = build_wide_json(1000);
    let total = sizeof_val(&wide);

    c.bench_function("exceeds_size early_exit", |b| {
        // Limit far below actual size — should short-circuit quickly
        b.iter(|| exceeds_size(black_box(&wide), 100))
    });
    c.bench_function("exceeds_size full_traverse", |b| {
        // Limit above actual size — must traverse everything
        b.iter(|| exceeds_size(black_box(&wide), total + 1000))
    });
}

criterion_group!(benches, bench_sizeof, bench_exceeds, bench_breakdown);
criterion_main!(benches);