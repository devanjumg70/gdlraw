use chromenet::urlrequest::request::URLRequest;
// use http_body_util::BodyExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Step 1: Setting cookie ---");
    let url1 = "https://httpbin.org/cookies/set?test_cookie=hello_chromenet";
    let mut req1 = URLRequest::new(url1)?;
    req1.start().await?;

    if let Some(resp) = req1.get_response() {
        println!("Step 1 Status: {}", resp.status());
        println!("Step 1 Headers: {:#?}", resp.headers());
        // Cookies are set in response headers or via logic.
    }

    println!("\n--- Step 2: Verifying cookie ---");
    let url2 = "https://httpbin.org/cookies";
    let mut req2 = URLRequest::new(url2)?;
    req2.start().await?;

    if let Some(resp) = req2.get_response() {
        println!("Step 2 Status: {}", resp.status());

        // Read body to see if cookie was sent back
        // let body = resp.body(); // This consumes the body if we could access it mutably, but get_response returns ref.
        // We need to implement body reading better in URLRequest if we want to consume it here.
        // For now, let's rely on the fact that we can't easily read the body ref without consuming it from the job.
        // Accessing Incoming directly:
        // We can't mutable access body via `get_response()` which gives `&Response`.

        println!("Response Headers: {:#?}", resp.headers());
        println!("(Check debug output for 'Cookie:' header sent or check if httpbin returns it if we could read body)");
    }

    Ok(())
}
