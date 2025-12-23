use chromenet::urlrequest::device::DeviceRegistry;
use chromenet::urlrequest::request::URLRequest;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_device_lookup(c: &mut Criterion) {
    c.bench_function("device_lookup", |b| {
        b.iter(|| black_box(DeviceRegistry::get_by_title("Pixel 7")))
    });
}

fn benchmark_set_device(c: &mut Criterion) {
    let device = DeviceRegistry::get_by_title("Pixel 7").unwrap();
    let url = "https://example.com";

    c.bench_function("set_device_overhead", |b| {
        b.iter(|| {
            let mut req = URLRequest::new(url).unwrap();
            req.set_device(device.clone());
            black_box(req)
        })
    });
}

criterion_group!(benches, benchmark_device_lookup, benchmark_set_device);
criterion_main!(benches);
