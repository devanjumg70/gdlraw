// use chromenet::cookies::canonical_cookie::CanonicalCookie;
use chromenet::cookies::decrypt;
use chromenet::cookies::monster::CookieMonster;
use chromenet::cookies::oscrypt;
use chromenet::cookies::safari;
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

fn benchmark_key_derivation(c: &mut Criterion) {
    c.bench_function("pbkdf2_derive_key_1iter", |b| {
        b.iter(|| {
            black_box(decrypt::derive_key(black_box(b"peanuts"), 1));
        })
    });

    c.bench_function("pbkdf2_derive_key_1003iter", |b| {
        b.iter(|| {
            black_box(decrypt::derive_key(black_box(b"peanuts"), 1003));
        })
    });
}

fn benchmark_v10_decryption(c: &mut Criterion) {
    // Create a fake encrypted cookie (v10 prefix + 16 bytes ciphertext)
    let mut encrypted = Vec::new();
    encrypted.extend_from_slice(b"v10");
    encrypted.extend_from_slice(&[0u8; 16]); // Dummy ciphertext

    c.bench_function("v10_decrypt_attempt", |b| {
        b.iter(|| {
            black_box(oscrypt::decrypt_v10(black_box(&encrypted)));
        })
    });
}

fn benchmark_safari_parse(c: &mut Criterion) {
    // Create a minimal valid Safari cookies file (header only, 0 pages)
    let mut data = Vec::new();
    data.extend_from_slice(b"cook");
    data.extend_from_slice(&0u32.to_be_bytes()); // 0 pages

    c.bench_function("safari_parse_empty", |b| {
        b.iter(|| {
            let _ = black_box(safari::parse_binary_cookies(black_box(&data)));
        })
    });
}

criterion_group!(
    benches,
    benchmark_cookie_insert,
    benchmark_cookie_get,
    benchmark_key_derivation,
    benchmark_v10_decryption,
    benchmark_safari_parse
);
criterion_main!(benches);
