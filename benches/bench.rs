use criterion::{criterion_group, criterion_main, Criterion};
use dhat_pprof::{convert, convert_profile, write_pprof, DhatProfile};
use std::hint::black_box;

fn conversion_benchmark(c: &mut Criterion) {
    // Read the test data once
    let json_content = std::fs::read_to_string("examples/dhat-heap.json")
        .expect("examples/dhat-heap.json not found - run 'cargo run --example automated --features dhat-heap' first");

    // Benchmark JSON parsing
    c.bench_function("parse_dhat_json", |b| {
        b.iter(|| {
            let profile: DhatProfile = serde_json::from_str(black_box(&json_content)).unwrap();
            black_box(profile);
        })
    });

    // Benchmark DHAT to pprof conversion (without I/O)
    let dhat_profile: DhatProfile = serde_json::from_str(&json_content).unwrap();
    c.bench_function("convert_to_pprof", |b| {
        b.iter(|| {
            let profile = convert_profile(black_box(dhat_profile.clone())).unwrap();
            black_box(profile);
        })
    });

    // Benchmark pprof encoding and compression
    let dhat_profile: DhatProfile = serde_json::from_str(&json_content).unwrap();
    let pprof_profile = convert_profile(dhat_profile).unwrap();

    // Ensure target directory exists
    std::fs::create_dir_all("target/dhat-pprof").unwrap();

    c.bench_function("write_pprof_gz", |b| {
        b.iter(|| {
            write_pprof(black_box(&pprof_profile), "target/dhat-pprof/bench-output.pb.gz").unwrap();
        })
    });

    // Benchmark full end-to-end conversion
    c.bench_function("convert_full_e2e", |b| {
        b.iter(|| {
            convert(
                black_box("examples/dhat-heap.json"),
                black_box("target/dhat-pprof/bench-output.pb.gz"),
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, conversion_benchmark);
criterion_main!(benches);
