//! Browser profiles for emulation.
//!
//! Contains predefined profiles for Chrome, Firefox, Safari, Edge, OkHttp, and Opera.

pub mod chrome;
pub mod edge;
pub mod firefox;
pub mod okhttp;
pub mod opera;
pub mod safari;

pub use chrome::Chrome;
pub use edge::Edge;
pub use firefox::Firefox;
pub use okhttp::OkHttp;
pub use opera::Opera;
pub use safari::Safari;
