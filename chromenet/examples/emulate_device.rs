use chromenet::urlrequest::device::DeviceRegistry;
use chromenet::urlrequest::request::URLRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.google.com/";
    let mut request = URLRequest::new(url)?;

    // Emulate Pixel 7
    if let Some(device) = DeviceRegistry::get_by_title("Pixel 7") {
        println!("Emulating: {}", device.title);
        request.set_device(device);
    } else {
        eprintln!("Device not found!");
        return Ok(());
    }

    request.start().await?;

    if let Some(response) = request.get_response() {
        println!("Status: {}", response.status());
        println!("Headers: {:#?}", response.headers());
        // We can't easily see the *sent* headers here without logging or a loopback server.
        // But success implies the request was well-formed.
        // We might check if google returned mobile content (vary: User-Agent etc)
    } else {
        println!("Request failed or no response");
    }

    Ok(())
}
