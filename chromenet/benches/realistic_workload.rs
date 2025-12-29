use chromenet::urlrequest::request::URLRequest;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn bench_scraping_pattern(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // We use a high sample count to get reliable distribution, but fewer iterations per sample if it's slow.
    // However, criterion handles that.
    
    let mut group = c.benchmark_group("scraping");
    group.sample_size(10); // Reduce sample size as network tests are slow

    group.bench_function("scrape_10_pages_same_domain", |b| {
        b.to_async(&rt).iter(|| async {
            // Note: In chromenet, the URLRequestContext (Pool/Factory) is global and lazy-loaded.
            // This benchmark effectively measures the performance of reused connections (pooling)
            // after the first iteration warms up the globals.
            
            for i in 1..=10 {
                let url = format!("https://httpbin.org/delay/0?page={}", i);
                // Use URLRequest::new, handling the Result with unwrap for the benchmark
                let mut req = URLRequest::new(&url).expect("Invalid URL");
                black_box(req.start().await).expect("Request failed");
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_scraping_pattern);
criterion_main!(benches);
