use boring::ssl::{SslConnector, SslMethod};
use chromenet::socket::tls::TlsConfig;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_tls_config_apply(c: &mut Criterion) {
    let config = TlsConfig::default_chrome();

    c.bench_function("tls_config_apply", |b| {
        b.iter(|| {
            let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
            config.apply_to_builder(&mut builder).unwrap();
            black_box(());
        })
    });
}

criterion_group!(benches, benchmark_tls_config_apply);
criterion_main!(benches);
