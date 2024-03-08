use criterion::{black_box, BenchmarkId, Criterion};
use docbuf_core::{
    traits::{DocBuf, DocBufMap},
    vtable::VTable,
};
use docbuf_tests::{complex, test_deps::*};

// Benchmark the serialization of a complex document.
pub fn benchmark_complex_deserialization_docbuf_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_complex_deserialization_docbuf_map");

    // Create an instance of a document
    let doc = complex::Document::dummy();

    // Create a buffer to serialize document into
    let mut buffer = Vec::with_capacity(1024);

    let offsets = doc
        .to_docbuf(&mut buffer)
        .expect("failed to serialize docbuf");

    let vtable = complex::Document::vtable().expect("failed to return vtable");

    let bincode_buffer = bincode::serialize(&doc).expect("failed to serialize bincode");

    group.bench_with_input(
        BenchmarkId::new(
            "DocBuf Map Deserialization",
            format!("{:?}", buffer.clone()),
        ),
        &(&vtable, &buffer.clone(), &offsets),
        |b, (vtable, buf, off)| {
            b.iter(|| {
                black_box(
                    // Return the signature from the buffer using the offsets and the vtable
                    <&VTable<'static> as DocBufMap<Vec<u8>>>::docbuf_map(vtable, buf, &off[4])
                        .expect("failed to deserialize docbuf"),
                )
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("DocBuf Deserialization", format!("{:?}", buffer.clone())),
        &buffer,
        |b, buffer| {
            b.iter(|| {
                black_box(
                    complex::Document::from_docbuf(&mut buffer.clone())
                        .expect("Failed to deserialize docbuf bytecode")
                        .metadata
                        .signature
                        .signature,
                )
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Bincode Deserialization", format!("{:?}", buffer.clone())),
        &bincode_buffer,
        |b, buffer| {
            b.iter(|| {
                black_box(
                    bincode::deserialize::<complex::Document>(buffer)
                        .expect("Failed to deserialize bincode")
                        .metadata
                        .signature
                        .signature,
                )
            })
        },
    );

    group.finish();
}
