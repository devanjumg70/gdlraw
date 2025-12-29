use chromenet::cookies::psl::is_public_suffix;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_psl_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("psl_lookup");

    // Test domains
    let domains = vec![
        "com",
        "co.uk",
        "github.io",
        "example.com",
        "google.com",
        "sub.example.com",
    ];

    group.bench_function("lookup_1000_mixed_domains", |b| {
        b.iter(|| {
            // Check 1000 times (approx 166 iter of the vec)
            for _ in 0..166 {
                for domain in &domains {
                    black_box(is_public_suffix(domain));
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_psl_lookup);
criterion_main!(benches);
