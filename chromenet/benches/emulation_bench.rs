//! Benchmarks for emulation module.

use chromenet::emulation::profiles::chrome::Chrome;
use chromenet::emulation::{Emulation, EmulationFactory, Http2Options};
use chromenet::socket::tls::TlsOptions;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_tls_options_creation(c: &mut Criterion) {
    c.bench_function("TlsOptions::default", |b| {
        b.iter(|| black_box(TlsOptions::default()))
    });

    c.bench_function("TlsOptions::builder full config", |b| {
        b.iter(|| {
            black_box(
                TlsOptions::builder()
                    .cipher_list("TLS_AES_128_GCM_SHA256")
                    .curves_list("X25519:P-256:P-384")
                    .sigalgs_list("ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256")
                    .grease_enabled(true)
                    .permute_extensions(true)
                    .session_ticket(true)
                    .build(),
            )
        })
    });
}

fn bench_emulation_creation(c: &mut Criterion) {
    c.bench_function("Emulation::default", |b| {
        b.iter(|| black_box(Emulation::default()))
    });

    c.bench_function("Emulation::builder with TLS", |b| {
        b.iter(|| {
            black_box(
                Emulation::builder()
                    .tls_options(TlsOptions::default())
                    .build(),
            )
        })
    });

    c.bench_function("Emulation::builder full config", |b| {
        b.iter(|| {
            black_box(
                Emulation::builder()
                    .tls_options(TlsOptions::default())
                    .http2_options(
                        Http2Options::builder()
                            .initial_window_size(6291456)
                            .max_header_list_size(262144)
                            .build(),
                    )
                    .build(),
            )
        })
    });
}

fn bench_chrome_profiles(c: &mut Criterion) {
    c.bench_function("Chrome::V140.emulation()", |b| {
        b.iter(|| black_box(Chrome::V140.emulation()))
    });

    c.bench_function("Chrome::V120.emulation()", |b| {
        b.iter(|| black_box(Chrome::V120.emulation()))
    });
}

fn bench_emulation_into_parts(c: &mut Criterion) {
    c.bench_function("Emulation::into_parts", |b| {
        b.iter(|| {
            let emu = Chrome::V140.emulation();
            black_box(emu.into_parts())
        })
    });
}

criterion_group!(
    benches,
    bench_tls_options_creation,
    bench_emulation_creation,
    bench_chrome_profiles,
    bench_emulation_into_parts,
);
criterion_main!(benches);
