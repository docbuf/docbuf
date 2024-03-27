use criterion::{black_box, BenchmarkId, Criterion};
use docbuf_core::{
    traits::{DocBuf, DocBufMap},
    // vtable::VTable,
};
use docbuf_tests::{complex, test_deps::*};

// Benchmark the serialization of a complex document.
pub fn benchmark_complex_roundtrip_field_mutation(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_complex_roundtrip_field_mutation");

    // Create an instance of a document
    let doc = complex::Document::dummy();

    group.bench_with_input(
        BenchmarkId::new("DocBuf Map Roundtrip", format!("{:?}", "")),
        &doc,
        |b, doc| {
            b.iter(|| {
                black_box({
                    let mut buffer = Vec::with_capacity(1024);

                    let mut offsets = doc
                        .to_docbuf(&mut buffer)
                        .expect("failed to serialize docbuf");

                    let vtable = complex::Document::vtable().expect("failed to return vtable");

                    let sig_offset = offsets.as_ref()[4].clone();

                    // Return the signature from the buffer using the offsets and the vtable
                    let _sig: Vec<u8> = vtable
                        .docbuf_map(&buffer, &sig_offset)
                        .expect("failed to deserialize docbuf");

                    let sig = vec![1; 32];

                    vtable.docbuf_map_replace(&sig, sig_offset, &mut buffer, &mut offsets)
                })
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("DocBuf Roundtrip", format!("{:?}", "")),
        &doc,
        |b, doc| {
            b.iter(|| {
                black_box({
                    let mut buffer = Vec::with_capacity(1024);

                    doc.to_docbuf(&mut buffer)
                        .expect("failed to serialize docbuf");

                    let mut doc2 = complex::Document::from_docbuf(&mut buffer.clone())
                        .expect("Failed to deserialize docbuf bytecode");

                    doc2.metadata.signature.signature = [1; 32];
                })
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Bincode Roundtrip", format!("{:?}", "")),
        &doc,
        |b, doc| {
            b.iter(|| {
                black_box({
                    let buffer = bincode::serialize(&doc).expect("failed to serialize bincode");

                    let mut doc2 = bincode::deserialize::<complex::Document>(&buffer)
                        .expect("Failed to deserialize bincode");

                    doc2.metadata.signature.signature = [1; 32];

                    bincode::serialize(&doc2).expect("failed to serialize bincode")
                })
            })
        },
    );

    group.finish();
}
