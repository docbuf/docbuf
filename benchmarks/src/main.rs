pub mod benches;

use criterion::{criterion_group, criterion_main};

pub use benches::*;

criterion_group!(
    benches,
    benchmark_complex_serialization,
    benchmark_complex_vtable,
    benchmark_unsigned_integers_serialization
);

criterion_main!(benches);
