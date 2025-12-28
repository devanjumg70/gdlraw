//! Base types and error handling.
//!
//! Provides foundational types mirroring Chromium's `net/base/`:
//! - [`NetError`]: Network error codes matching `net_error_list.h`
//! - [`LoadState`]: Request loading states from `load_states_list.h`

pub mod context;
pub mod loadstate;
pub mod neterror;

#[cfg(test)]
mod tests;
