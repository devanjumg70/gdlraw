// use chromenet::socket::proxy::ProxySettings;
// use chromenet::urlrequest::request::URLRequest;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // NOTE: This example requires a running proxy server.
    // If you have one, un-comment and configure below.
    // Otherwise it will look like a failure or we can point to a mock.

    // Example: HTTP Proxy at localhost:8080
    // let proxy_url = "http://127.0.0.1:8080";

    // let mut req = URLRequest::new("https://httpbin.org/ip")?;

    // if let Some(p) = ProxySettings::new(proxy_url) {
    //     println!("Setting proxy to: {}", proxy_url);
    //     req.set_proxy(p);
    // } else {
    //     println!("Invalid proxy url");
    //     return Ok(());
    // }

    // println!("--- Starting Proxy Request ---");
    // match req.start().await {
    //     Ok(_) => {
    //         println!("Status: {}", req.get_response().unwrap().status());
    //         // In a real test we would read body to see if the IP matches expected proxy IP
    //     }
    //     Err(e) => println!("Error: {}", e),
    // }

    println!("Proxy example created. Configure a proxy in the code to test.");
    Ok(())
}
