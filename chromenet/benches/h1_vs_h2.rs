use chromenet::urlrequest::request::URLRequest;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use url::Url;

// Note: For real H1 vs H2 benchmarking, we need a server that supports both.
// Doing it against google.com is flaky and bad practice for CI.
// Setting up a local H2 server with TLS certificates in a benchmark is complex.
//
// For this "Clean & Raw" exercise, we will benchmark the *Client Logic* overhead
// assuming a mock socket or just measuring the Transaction setup time.
//
// However, since we want to demonstrate H2 multiplexing, we really need a server.
//
// Given limitations, we will placeholder this benchmark with a note that
// it requires a local test server capable of ALPN.

fn benchmark_transaction_setup(c: &mut Criterion) {
    let _rt = Runtime::new().unwrap();
    let _url = Url::parse("https://www.google.com").unwrap(); // We'll just init, not start

    c.bench_function("transaction_new", |b| {
        b.iter(|| {
            let req = URLRequest::new(black_box("https://www.google.com")).unwrap();
            black_box(req);
        })
    });
}

criterion_group!(benches, benchmark_transaction_setup);
criterion_main!(benches);
