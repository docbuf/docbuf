use criterion::{black_box, Criterion};
use docbuf_core::{traits::DocBuf, vtable::VTable};
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

pub fn benchmark_vtable_struct_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_vtable_struct_lookup");

    let vtable = complex::Document::vtable().expect("Failed to create vtable");

    group.bench_with_input(
        "benchmark_vtable_struct_lookup_by_name",
        &vtable,
        |b, vtable| {
            b.iter(|| {
                black_box(
                    vtable
                        .struct_by_name("Document")
                        .expect("Failed to find struct"),
                )
            })
        },
    );

    group.bench_with_input(
        "benchmark_vtable_struct_lookup_by_index",
        &vtable,
        |b, vtable| b.iter(|| black_box(vtable.struct_by_index(2).expect("Failed to find struct"))),
    );

    group.finish();
}
