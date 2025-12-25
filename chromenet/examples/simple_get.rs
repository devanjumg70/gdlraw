//! Simple HTTP GET request example.
//!
//! This example demonstrates the core networking components.

use chromenet::cookies::monster::CookieMonster;
use chromenet::http::streamfactory::HttpStreamFactory;
use chromenet::http::transaction::HttpNetworkTransaction;
use chromenet::socket::pool::ClientSocketPool;
use std::sync::Arc;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let pool = Arc::new(ClientSocketPool::new());
    let factory = Arc::new(HttpStreamFactory::new(pool));
    let cookies = Arc::new(CookieMonster::new());

    // Create HTTP transaction
    let url = Url::parse("https://httpbin.org/get")?;
    let mut transaction = HttpNetworkTransaction::new(factory, url, cookies);

    // Execute the request
    println!("Sending request to httpbin.org...");
    transaction.start().await?;

    // Get the response
    if let Some(response) = transaction.get_response() {
        println!("Status: {}", response.status());
        println!("Headers:");
        for (name, value) in response.headers() {
            println!("  {}: {:?}", name, value);
        }
    }

    Ok(())
}
