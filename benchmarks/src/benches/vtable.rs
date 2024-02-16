use criterion::{black_box, Criterion};
use docbuf_core::{
    traits::DocBuf,
    vtable::{FieldRules, VTable},
};
use docbuf_tests::complex;

// Benchmark the creation of the vtable for a complex document.
pub fn benchmark_complex_vtable(c: &mut Criterion) {
    c.bench_function("benchmark_complex_vtable", |b| {
        b.iter(|| black_box(complex::Document::vtable()))
    });
}

pub fn benchmark_new_vtable(c: &mut Criterion) {
    c.bench_function("benchmark_new_vtable", |b| {
        b.iter(|| black_box(VTable::new()))
    });
}

pub fn benchmark_le_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_le_bytes");

    group.bench_function("benchmark_u16_bytes", |b| {
        b.iter(|| black_box(FieldRules::le_bytes_data_length(u16::MAX as usize)))
    });

    group.bench_function("benchmark_u64_le_bytes", |b| {
        b.iter(|| black_box(FieldRules::le_bytes_data_length(u64::MAX as usize)))
    });

    group.finish();
}
