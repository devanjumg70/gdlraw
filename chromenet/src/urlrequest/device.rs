// use crate::base::neterror::NetError;

#[derive(Debug, Clone)]
pub struct Device {
    pub title: &'static str,
    pub user_agent: &'static str,
    pub user_agent_metadata: Option<UserAgentMetadata>,
    pub screen: Screen,
    pub capabilities: &'static [&'static str], // e.g., "touch", "mobile"
}

#[derive(Debug, Clone)]
pub struct UserAgentMetadata {
    pub platform: &'static str,
    pub platform_version: &'static str,
    pub architecture: &'static str,
    pub model: &'static str,
    pub mobile: bool,
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f32,
    pub horizontal: Orientation, // Simplify: just store dimensions for now? Or full orientation?
    pub vertical: Orientation,
}

#[derive(Debug, Clone)]
pub struct Orientation {
    pub width: u32,
    pub height: u32,
}

// Registry of devices ported from Chromium's EmulatedDevices.ts
pub struct DeviceRegistry;

impl DeviceRegistry {
    pub fn get_by_title(title: &str) -> Option<Device> {
        Self::all().into_iter().find(|d| d.title == title)
    }

    pub fn all() -> Vec<Device> {
        vec![
            // iPhone 12 Pro
            Device {
                title: "iPhone 12 Pro",
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3 Mobile/15E148 Safari/604.1",
                user_agent_metadata: None, // Apple devices often don't send full CH in same way, or it's not in the TS file for this one?
                // Wait, TS file has "iPhone 12 Pro" with "user-agent-metadata": undefined in my preview?
                // Let's re-read the file carefully. The "iPhone 12 Pro" entry in TS file (lines 708-727) does NOT have user-agent-metadata.
                // It seems newer Chrome/Pixel devices have it.
                screen: Screen {
                    width: 390,
                    height: 844,
                    device_scale_factor: 3.0,
                    horizontal: Orientation { width: 844, height: 390 },
                    vertical: Orientation { width: 390, height: 844 },
                },
                capabilities: &["touch", "mobile"],
            },
            // Pixel 7
            Device {
                title: "Pixel 7",
                user_agent: "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/%s Mobile Safari/537.36",
                // Note: %s needs to be replaced with Chrome version.
                user_agent_metadata: Some(UserAgentMetadata {
                    platform: "Android",
                    platform_version: "13",
                    architecture: "",
                    model: "Pixel 7",
                    mobile: true,
                }),
                screen: Screen {
                    width: 412,
                    height: 915,
                    device_scale_factor: 2.625,
                    horizontal: Orientation { width: 915, height: 412 },
                    vertical: Orientation { width: 412, height: 915 },
                },
                capabilities: &["touch", "mobile"],
            },
            // Samsung Galaxy S8+
            Device {
                title: "Samsung Galaxy S8+",
                user_agent: "Mozilla/5.0 (Linux; Android 8.0.0; SM-G955U Build/R16NW) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/%s Mobile Safari/537.36",
                user_agent_metadata: Some(UserAgentMetadata {
                    platform: "Android",
                    platform_version: "8.0.0",
                    architecture: "",
                    model: "SM-G955U",
                    mobile: true,
                }),
                screen: Screen {
                    width: 360,
                    height: 740,
                    device_scale_factor: 4.0,
                    horizontal: Orientation { width: 740, height: 360 },
                    vertical: Orientation { width: 360, height: 740 },
                },
                capabilities: &["touch", "mobile"],
            },
        ]
    }
}
