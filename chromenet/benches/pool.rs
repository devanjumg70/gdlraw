use chromenet::socket::pool::ClientSocketPool;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use url::Url;

/// Benchmark pool creation and limit checking overhead.
/// These are pure in-memory operations that don't require network I/O.
fn benchmark_pool_operations(c: &mut Criterion) {
    // Benchmark pool creation
    c.bench_function("pool_new", |b| {
        b.iter(|| black_box(ClientSocketPool::new()))
    });

    // Benchmark pool statistics (pure memory operations)
    let pool = ClientSocketPool::new();
    let url = Url::parse("https://example.com").unwrap();

    c.bench_function("pool_stats", |b| {
        b.iter(|| {
            let _ = black_box(pool.total_active_count());
            let _ = black_box(pool.pending_request_count(&url));
        })
    });

    // Benchmark idle socket count (pure memory op)
    c.bench_function("pool_idle_socket_count", |b| {
        b.iter(|| black_box(pool.idle_socket_count()))
    });
}

criterion_group!(benches, benchmark_pool_operations);
criterion_main!(benches);
