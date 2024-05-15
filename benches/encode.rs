use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rand::RngCore;

fn create_source_block_data(length: usize) -> Vec<u8> {
    let mut output = vec![0u8; length];

    // Random buffer
    let mut rng = rand::thread_rng();
    rng.fill_bytes(output.as_mut());

    output
}

fn raptor_benchmark(c: &mut Criterion) {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::builder().is_test(true).try_init().ok();

    let data = create_source_block_data(100 * 1024 * 1024);

    let mut group = c.benchmark_group("encode");
    group.throughput(Throughput::Bytes(1000));
    group.bench_function("encode 1k", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(black_box(&data[..1000]), black_box(1), black_box(2))
        })
    });

    group.throughput(Throughput::Bytes(10 * 1024));
    group.bench_function("encode 10k", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(
                black_box(&data[0..10 * 1024]),
                black_box(10),
                black_box(10),
            )
        })
    });

    group.throughput(Throughput::Bytes(100 * 1024));
    group.bench_function("encode 100k", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(
                black_box(&data[0..100 * 1024]),
                black_box(100),
                black_box(100),
            )
        })
    });

    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("encode 1MB", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(
                black_box(&data[0..1024 * 1024]),
                black_box(1024),
                black_box(1024),
            )
        })
    });

    group.throughput(Throughput::Bytes(4 * 1024 * 1024));
    group.bench_function("encode 4MB", |b| {
        b.iter(|| {
            // for _ in 0..1000 {
            //     raptor_code::encode_source_block(
            //         black_box(&data[0..1024 * 1024]),
            //         black_box(64),
            //         black_box(10),
            //         );
            // }
            let (symbols, _) = raptor_code::encode_source_block(
                black_box(&data[0..4 * 1024 * 1024]),
                black_box(4 * 1024),
                black_box(4 * 1024),
            );
            assert_eq!(symbols.len(), 2 * 4 * 1024);
            assert!(symbols[0].len() > 1000);
            assert!(symbols[0].len() < 2000);
        })
    });

    // group.throughput(Throughput::Bytes(10 * 1024 * 1024));
    // group.bench_function("encode 10MB", |b| {
    //     b.iter(|| {
    //         raptor_code::encode_source_block(
    //             black_box(&data[0..10 * 1024 * 1024]),
    //             black_box(64),
    //             black_box(10),
    //         );
    //     })
    // });
}

criterion_group!(benches, raptor_benchmark);
criterion_main!(benches);
