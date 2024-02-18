pub mod benches;

use criterion::{criterion_group, criterion_main};

pub use benches::*;

criterion_group!(
    // Name of the benchmark group.
    // See use in the `criterion_main!` macro
    docbuf_benchmarks,
    // VTable Benchmarks
    vtable::benchmark_complex_vtable,
    vtable::benchmark_new_vtable,
    vtable::benchmark_le_bytes,
    vtable::benchmark_vtable_struct_lookup,
    // Serialization Benchmarks
    // serialization::benchmark_docbuf_serializer,
    serialization::benchmark_complex_serialization,
    serialization::benchmark_unsigned_integers,
);

criterion_main!(docbuf_benchmarks);
