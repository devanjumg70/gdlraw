use chromenet::client::Client;
use chromenet::cookies::monster::CookieMonster;
use chromenet::emulation::profiles::chrome::Chrome;
use chromenet::http::multipart::{Form, Part};
use chromenet::socket::proxy::ProxyBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Advanced Cookie Management
    // Create a persistent cookie jar that we can inspect later
    let jar = CookieMonster::new();
    
    // Pre-load a session cookie (simulating a saved login state)
    // In a real app, you might load this from a file using `jar.import_netscape(...)`
    jar.parse_and_save_cookie(
        &url::Url::parse("https://httpbin.org")?,
        "session_id=xyz123; Domain=httpbin.org; Path=/; Secure; HttpOnly"
    );

    // 2. Complex Proxy Configuration
    // Define a primary SOCKS5 proxy with authentication
    let primary_proxy = ProxyBuilder::new()
        .socks5("127.0.0.1:9050")
        .auth("user", "password")
        .no_proxy("localhost,127.0.0.1,internal.corp") // Bypass limits
        .build()
        .expect("Invalid proxy config");

    // 3. Client Construction with Browser Emulation
    let client = Client::builder()
        // Use a modern Chrome profile for TLS fingerprinting and header ordering
        .emulation(Chrome::V143) 
        // Share the cookie jar
        .cookie_store(jar.clone())
        // Set the proxy
        .proxy(primary_proxy)
        // Set a global timeout
        .timeout(Duration::from_secs(30))
        .build();

    println!("🤖 Client initialized with Chrome V143 fingerprint");

    // 4. Multipart File Upload
    // Create a complex multipart body with a file and metadata
    let file_content = b"Hello, this is a distinct file.";
    let form = Form::new()
        .text("upload_type", "standard")
        .text("user_ref", "8842")
        .part(
            "document", 
            Part::bytes(file_content.as_slice())
                .file_name("report.txt")
                .content_type("text/plain")
        );

    let content_type = form.content_type();
    let body = form.into_body();

    // 5. Build and Send Request with Overrides
    println!("🚀 Sending upload request to httpbin.org...");
    
    // Note: This request will fail if you don't actually have a SOCKS proxy at 127.0.0.1:9050
    // But it demonstrates the API usage perfectly.
    let request = client.post("https://httpbin.org/post")
        // Override the global Chrome emulation for this specific request?
        // Let's pretend we want to look like a mobile device for this one call:
        // .emulation(Safari::Ios17) 
        
        // Add custom headers (these are merged with emulation headers)
        .header("X-Custom-ID", "999")
        .header("Content-Type", content_type) // Crucial for multipart
        
        // Attach the body
        .body(body);

    // Execute
    match request.send().await {
        Ok(response) => {
            println!("✅ Response Status: {}", response.status());
            
            // 6. Response Handling
            // We can stream the body or get it as text/json
            let text = response.text().await?;
            println!("📄 Response Body Preview: {:.100}...", text);
            
            // 7. Inspect Cookies (Post-Request)
            // Any Set-Cookie headers from the server are now in our `jar`
            let cookies = jar.get_cookies_for_url(&url::Url::parse("https://httpbin.org")?);
            println!("🍪 Cookies after request: {}", cookies.len());
            for cookie in cookies {
                println!("   - {} = {}", cookie.name, cookie.value);
            }
        },
        Err(e) => {
            println!("❌ Request failed (expected if no proxy): {}", e);
            // This error print is expected since 127.0.0.1:9050 likely doesn't exist
        }
    }

    Ok(())
}
