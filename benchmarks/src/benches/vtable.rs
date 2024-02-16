use criterion::{black_box, BenchmarkId, Criterion};
use docbuf_core::{
    traits::DocBuf,
    vtable::{FieldRules, VTable},
};
use docbuf_tests::{complex, test_deps::*, unsigned_integers::*};

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

    // // Create an instance of a document
    // let doc = complex::Document::dummy();

    // group.bench_with_input(BenchmarkId::new("DocBuf", doc.clone()), &doc, |b, doc| {
    //     b.iter(|| black_box(doc.to_docbuf().expect("Failed to serialize docbuf")))
    // });

    // group.bench_with_input(BenchmarkId::new("Bincode", doc.clone()), &doc, |b, doc| {
    //     b.iter(|| black_box(bincode::serialize(doc).expect("Failed to serialize bincode")))
    // });

    // group.bench_with_input(BenchmarkId::new("JSON", doc.clone()), &doc, |b, doc| {
    //     b.iter(|| black_box(serde_json::to_vec(doc).expect("Failed to serialize json")))
    // });

    group.finish();
}
