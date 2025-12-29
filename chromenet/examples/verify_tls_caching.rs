use chromenet::urlrequest::request::URLRequest;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://httpbin.org/get";

    println!("=== Cold Start (First Request) ===");
    let start = Instant::now();
    let mut req = URLRequest::new(url)?;
    req.start().await?;
    println!("First request: {:?}", start.elapsed());
    
    println!("\n=== Warm Start (Reused Connection) ===");
    for i in 1..=5 {
        let start = Instant::now();
        let mut req = URLRequest::new(url)?;
        req.start().await?;
        println!("Request {}: {:?}", i, start.elapsed());
    }
    
    Ok(())
}
