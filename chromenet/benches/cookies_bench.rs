// use chromenet::cookies::canonical_cookie::CanonicalCookie;
use chromenet::cookies::monster::CookieMonster;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use url::Url;

fn benchmark_cookie_insert(c: &mut Criterion) {
    let store = CookieMonster::new();
    let url = Url::parse("https://example.com").unwrap();

    c.bench_function("cookie_parse_and_save", |b| {
        b.iter(|| {
            store.parse_and_save_cookie(black_box(&url), black_box("foo=bar; Path=/; Secure"));
        })
    });
}

fn benchmark_cookie_get(c: &mut Criterion) {
    let store = CookieMonster::new();
    let url = Url::parse("https://example.com/foo/bar").unwrap();
    // Pre-populate
    for i in 0..100 {
        store.parse_and_save_cookie(&url, &format!("cookie{}=val; Path=/foo", i));
    }

    c.bench_function("cookie_get_for_url", |b| {
        b.iter(|| {
            black_box(store.get_cookies_for_url(black_box(&url)));
        })
    });
}

criterion_group!(benches, benchmark_cookie_insert, benchmark_cookie_get);
criterion_main!(benches);
