//! Public Suffix List performance benchmark.

use chromenet::cookies::psl::{is_public_suffix, is_valid_cookie_domain, registrable_domain};
use criterion::{criterion_group, criterion_main, Criterion};

fn psl_lookups(c: &mut Criterion) {
    c.bench_function("psl_is_public_suffix_tld", |b| b.iter(|| is_public_suffix("com")));

    c.bench_function("psl_is_public_suffix_ccTLD", |b| b.iter(|| is_public_suffix("co.uk")));

    c.bench_function("psl_is_public_suffix_domain", |b| b.iter(|| is_public_suffix("example.com")));
}

fn psl_registrable_domain(c: &mut Criterion) {
    c.bench_function("psl_registrable_simple", |b| {
        b.iter(|| registrable_domain("www.example.com"))
    });

    c.bench_function("psl_registrable_ccTLD", |b| b.iter(|| registrable_domain("www.bbc.co.uk")));
}

fn psl_cookie_validation(c: &mut Criterion) {
    c.bench_function("psl_valid_cookie_domain", |b| {
        b.iter(|| is_valid_cookie_domain("www.example.com", ".example.com"))
    });

    c.bench_function("psl_supercookie_rejection", |b| {
        b.iter(|| is_valid_cookie_domain("example.com", ".com"))
    });
}

criterion_group!(benches, psl_lookups, psl_registrable_domain, psl_cookie_validation);
criterion_main!(benches);
