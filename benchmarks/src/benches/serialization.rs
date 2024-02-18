use criterion::{black_box, BenchmarkId, Criterion};
use docbuf_core::{serde::ser::DocBufSerializer, traits::DocBuf};
use docbuf_tests::{complex, test_deps::*, unsigned_integers::*};

// Benchmark the serialization of a complex document.
pub fn benchmark_complex_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_complex_serialization");

    // Create an instance of a document
    let doc = complex::Document::dummy();

    // Create a buffer to serialize document into
    let mut buffer = Vec::with_capacity(1024);

    group.bench_with_input(BenchmarkId::new("DocBuf", doc.clone()), &doc, |b, doc| {
        b.iter(|| {
            black_box(
                doc.to_docbuf(&mut buffer)
                    .expect("Failed to serialize docbuf"),
            )
        })
    });

    group.bench_with_input(BenchmarkId::new("Bincode", doc.clone()), &doc, |b, doc| {
        b.iter(|| black_box(bincode::serialize(doc).expect("Failed to serialize bincode")))
    });

    group.bench_with_input(BenchmarkId::new("JSON", doc.clone()), &doc, |b, doc| {
        b.iter(|| black_box(serde_json::to_vec(doc).expect("Failed to serialize json")))
    });

    group.finish();
}

// Benchmark the serialization of a complex document.
pub fn benchmark_unsigned_integers(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_unsigned_integers");

    // Create an instance of a document
    let u8_value = U8Value { u8_value: u8::MAX };

    // Create a buffer to serialize document into
    let mut buffer = Vec::with_capacity(1024);

    group.bench_with_input(
        BenchmarkId::new("DocBuf", u8_value.clone()),
        &u8_value,
        |b, u8_value| {
            b.iter(|| {
                black_box(
                    u8_value
                        .to_docbuf(&mut buffer)
                        .expect("Failed to serialize docbuf"),
                )
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Bincode", u8_value.clone()),
        &u8_value,
        |b, u8_value| {
            b.iter(|| black_box(bincode::serialize(u8_value).expect("Failed to serialize bincode")))
        },
    );

    group.bench_with_input(
        BenchmarkId::new("JSON", u8_value.clone()),
        &u8_value,
        |b, u8_value| {
            b.iter(|| black_box(serde_json::to_vec(u8_value).expect("Failed to serialize json")))
        },
    );

    let u16_value = U16Value {
        u16_value: u16::MAX,
    };

    group.bench_with_input(
        BenchmarkId::new("DocBuf", u16_value.clone()),
        &u16_value,
        |b, u16_value| {
            b.iter(|| {
                black_box(
                    u16_value
                        .to_docbuf(&mut buffer)
                        .expect("Failed to serialize docbuf"),
                )
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Bincode", u16_value.clone()),
        &u16_value,
        |b, u16_value| {
            b.iter(|| {
                black_box(bincode::serialize(u16_value).expect("Failed to serialize bincode"))
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("JSON", u16_value.clone()),
        &u16_value,
        |b, u16_value| {
            b.iter(|| black_box(serde_json::to_vec(u16_value).expect("Failed to serialize json")))
        },
    );

    group.finish();
}

// pub fn benchmark_docbuf_serializer(c: &mut Criterion) {
//     let vtable = complex::Document::vtable().expect("Failed to create vtable");
//     let mut buffer = Vec::with_capacity(1024);

//     c.bench_with_input(
//         BenchmarkId::new("benchmark_docbuf_serializer", ""),
//         &(vtable, &mut buffer),
//         |b, (v, &mut buf)| b.iter(|| black_box(DocBufSerializer::new(v, &mut buf))),
//     );
// }
