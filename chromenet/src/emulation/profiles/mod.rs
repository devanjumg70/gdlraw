//! Browser profiles for emulation.
//!
//! Contains predefined profiles for Chrome, Firefox, Safari, and Edge.

pub mod chrome;
pub mod edge;
pub mod firefox;
pub mod safari;

pub use chrome::Chrome;
pub use edge::Edge;
pub use firefox::Firefox;
pub use safari::Safari;
