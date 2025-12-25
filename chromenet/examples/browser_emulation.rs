//! Browser emulation example.
//!
//! This example demonstrates how to use device profiles and HTTP/2
//! fingerprinting to emulate different browsers.

use chromenet::http::h2settings::H2Settings;
use chromenet::urlrequest::device::DeviceRegistry;

fn main() {
    // Get all available device profiles from the registry
    let devices = DeviceRegistry::all();

    println!("--- Available Device Profiles ---\n");
    for device in &devices {
        println!("{}", device.title);
        println!(
            "  Screen: {}x{} @ {}x",
            device.screen.width, device.screen.height, device.screen.device_scale_factor
        );
        println!("  Mobile: {}", device.is_mobile());
        println!("  Touch: {}", device.has_touch());
        println!();
    }

    // Look up a specific device by title
    if let Some(pixel) = DeviceRegistry::get_by_title("Pixel 7") {
        println!("--- Pixel 7 Details ---");
        println!("User-Agent: {}", pixel.get_user_agent("120.0.6099.71"));

        // Get Client Hints headers
        let hints = pixel.get_client_hint_headers("120");
        println!("\nClient Hints:");
        for (name, value) in hints {
            println!("  {}: {}", name, value);
        }
    }

    // HTTP/2 SETTINGS fingerprinting
    println!("\n--- HTTP/2 SETTINGS Presets ---\n");

    let chrome_settings = H2Settings::chrome();
    println!("Chrome:");
    println!("  Initial window size: {}", chrome_settings.initial_window_size);
    println!("  Max frame size: {}", chrome_settings.max_frame_size);
    println!("  Max concurrent streams: {}", chrome_settings.max_concurrent_streams);
    println!("  Header table size: {}", chrome_settings.header_table_size);
    println!("  Enable push: {}", chrome_settings.enable_push);

    let firefox_settings = H2Settings::firefox();
    println!("\nFirefox:");
    println!("  Initial window size: {}", firefox_settings.initial_window_size);
    println!("  Max frame size: {}", firefox_settings.max_frame_size);
    println!("  Max concurrent streams: {}", firefox_settings.max_concurrent_streams);

    let safari_settings = H2Settings::safari();
    println!("\nSafari:");
    println!("  Initial window size: {}", safari_settings.initial_window_size);
    println!("  Max frame size: {}", safari_settings.max_frame_size);
    println!("  Max concurrent streams: {}", safari_settings.max_concurrent_streams);

    // Show key differences for fingerprinting
    println!("\n--- Fingerprinting Differences ---");
    println!("Chrome initial_window_size: {} bytes", chrome_settings.initial_window_size);
    println!("Firefox initial_window_size: {} bytes", firefox_settings.initial_window_size);
    println!(
        "Difference: {} bytes",
        chrome_settings.initial_window_size as i64 - firefox_settings.initial_window_size as i64
    );
}
