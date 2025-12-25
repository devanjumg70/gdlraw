//! HSTS lookup performance benchmark.

use chromenet::tls::hsts::HstsStore;
use criterion::{criterion_group, criterion_main, Criterion};

fn hsts_lookup_preloaded(c: &mut Criterion) {
    let store = HstsStore::with_preload();

    c.bench_function("hsts_lookup_hit", |b| b.iter(|| store.should_upgrade("google.com")));

    c.bench_function("hsts_lookup_miss", |b| {
        b.iter(|| store.should_upgrade("unknown-domain-12345.com"))
    });

    c.bench_function("hsts_lookup_subdomain", |b| {
        b.iter(|| store.should_upgrade("mail.google.com"))
    });
}

fn hsts_add_dynamic(c: &mut Criterion) {
    c.bench_function("hsts_add_header", |b| {
        let store = HstsStore::new();
        b.iter(|| store.add_from_header("example.com", "max-age=31536000; includeSubDomains"))
    });
}

criterion_group!(benches, hsts_lookup_preloaded, hsts_add_dynamic);
criterion_main!(benches);
