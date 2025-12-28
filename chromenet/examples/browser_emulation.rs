//! Browser emulation example.
//!
//! This example demonstrates how to use device profiles and HTTP/2
//! fingerprinting to emulate different browsers.

use chromenet::http::H2Fingerprint;
use chromenet::urlrequest::device::DeviceRegistry;

fn main() {
    // Get all available device profiles from the registry
    let devices = DeviceRegistry::all();

    println!("--- Available Device Profiles ---\n");
    for device in devices {
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
    println!("\n--- HTTP/2 Fingerprint Presets ---\n");

    let chrome_fp = H2Fingerprint::chrome();
    println!("Chrome H2 Fingerprint:");
    println!("  Initial window size: {}", chrome_fp.initial_window_size);
    println!(
        "  Initial conn window: {}",
        chrome_fp.initial_conn_window_size
    );
    if let Some(max_frame) = chrome_fp.max_frame_size {
        println!("  Max frame size: {}", max_frame);
    }
    if let Some(max_concurrent) = chrome_fp.max_concurrent_streams {
        println!("  Max concurrent streams: {}", max_concurrent);
    }

    let firefox_fp = H2Fingerprint::firefox();
    println!("\nFirefox H2 Fingerprint:");
    println!("  Initial window size: {}", firefox_fp.initial_window_size);
    println!(
        "  Initial conn window: {}",
        firefox_fp.initial_conn_window_size
    );

    let safari_fp = H2Fingerprint::safari();
    println!("\nSafari H2 Fingerprint:");
    println!("  Initial window size: {}", safari_fp.initial_window_size);

    // Show key differences for fingerprinting
    println!("\n--- Fingerprinting Differences ---");
    println!(
        "Chrome initial_window_size: {} bytes",
        chrome_fp.initial_window_size
    );
    println!(
        "Firefox initial_window_size: {} bytes",
        firefox_fp.initial_window_size
    );
    println!(
        "Difference: {} bytes",
        chrome_fp.initial_window_size as i64 - firefox_fp.initial_window_size as i64
    );
}
