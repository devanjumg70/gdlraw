//! Certificate pinning performance benchmark.

use chromenet::tls::pinning::{PinSet, PinStore};
use criterion::{criterion_group, criterion_main, Criterion};

fn pinning_check(c: &mut Criterion) {
    let store = PinStore::new();

    // Add some pinned domains
    for i in 0..10 {
        let hash = [i as u8; 32];
        let mut pin_set = PinSet::new(format!("example{}.com", i));
        pin_set.add_pin(hash);
        store.add(pin_set);
    }

    let valid_hash = [5u8; 32];

    c.bench_function("pinning_check_hit", |b| {
        b.iter(|| store.check("example5.com", &[valid_hash]))
    });

    c.bench_function("pinning_check_miss", |b| {
        b.iter(|| store.check("unknown.com", &[valid_hash]))
    });
}

fn pinning_subdomain(c: &mut Criterion) {
    let store = PinStore::new();

    let hash = [42u8; 32];
    let mut pin_set = PinSet::new("example.com").include_subdomains(true);
    pin_set.add_pin(hash);
    store.add(pin_set);

    c.bench_function("pinning_subdomain_check", |b| {
        b.iter(|| store.check("deep.sub.example.com", &[hash]))
    });
}

criterion_group!(benches, pinning_check, pinning_subdomain);
criterion_main!(benches);
