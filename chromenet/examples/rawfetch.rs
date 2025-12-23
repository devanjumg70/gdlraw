use chromenet::urlrequest::request::URLRequest;
// use http_body_util::BodyExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Create a request
    let url = "https://www.google.com/";
    println!("Fetching {}", url);

    let mut request = URLRequest::new(url)?;

    // 2. Start (Connect -> Send -> Read Headers)
    request.start().await?;

    // 3. Inspect Response
    if let Some(response) = request.get_response() {
        println!("Status: {}", response.status());
        println!("Headers: {:#?}", response.headers());

        // 4. Read Body (For now, just dumping it if possible, but our current impl is minimal)
        // In a real impl, we'd read chunks.
        // For this MVP, we just successfully connected and got headers.
        println!("Successfully received success response!");
    } else {
        eprintln!("No response received.");
    }

    Ok(())
}
