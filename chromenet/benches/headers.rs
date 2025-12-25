use chromenet::http::orderedheaders::OrderedHeaderMap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_headers_to_header_map(c: &mut Criterion) {
    let mut headers = OrderedHeaderMap::new();
    headers.insert(
        "Accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"
    ).unwrap();
    headers
        .insert("Accept-Encoding", "gzip, deflate, br")
        .unwrap();
    headers.insert("Accept-Language", "en-GB,en;q=0.9").unwrap();
    headers.insert("Cache-Control", "max-age=0").unwrap();
    headers.insert(
        "Cookie",
        "WMF-Last-Access=xxxxxxxxxxx; WMF-Last-Access-Global=xxxxxxxxxxx; GeoIP=xxxxxxxxxxxxxxxxxxxxxxxxxxx; NetworkProbeLimit=0.001; enwikimwuser-sessionId=xxxxxxxxxxxxxxxxxxxx"
    ).unwrap();
    headers
        .insert(
            "Sec-Ch-Ua",
            "\"Google Chrome\";v=\"117\", \"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"117\"",
        )
        .unwrap();
    headers.insert("Sec-Ch-Ua-Mobile", "?0").unwrap();
    headers.insert("Sec-Ch-Ua-Platform", "\"Linux\"").unwrap();
    headers.insert("Sec-Fetch-Dest", "document").unwrap();
    headers.insert("Sec-Fetch-Mode", "navigate").unwrap();
    headers.insert("Sec-Fetch-Site", "none").unwrap();
    headers.insert("Sec-Fetch-User", "?1").unwrap();
    headers.insert("Upgrade-Insecure-Requests", "1").unwrap();
    headers.insert(
        "User-Agent",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36"
    ).unwrap();

    // Benchmark the cloning + conversion cost (simulating per-request overhead)
    c.bench_function("headers_to_header_map", |b| {
        b.iter(|| black_box(headers.clone()).to_header_map())
    });
}

fn benchmark_headers_insert(c: &mut Criterion) {
    c.bench_function("headers_insert", |b| {
        b.iter(|| {
            let mut headers = OrderedHeaderMap::new();
            headers.insert("Accept", "text/html").unwrap();
            headers.insert("User-Agent", "Mozilla/5.0").unwrap();
            headers.insert("Connection", "keep-alive").unwrap();
            black_box(headers)
        })
    });
}

criterion_group!(
    benches,
    benchmark_headers_to_header_map,
    benchmark_headers_insert
);
criterion_main!(benches);
