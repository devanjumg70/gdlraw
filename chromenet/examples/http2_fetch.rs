use chromenet::urlrequest::request::URLRequest;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Google supports H2
    let url = "https://www.google.com";
    println!("Fetching {}...", url);

    let mut req = URLRequest::new(url)?;

    match req.start().await {
        Ok(_) => {
            let resp = req.get_response().unwrap();
            println!("Status: {}", resp.status());
            println!("Version: {:?}", resp.version());

            // If H2 is working, version should be HTTP/2.0
            if resp.version() == http::Version::HTTP_2 {
                println!("SUCCESS: Negotiated HTTP/2!");
            } else {
                println!("WARNING: Negotiated {:?}", resp.version());
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
