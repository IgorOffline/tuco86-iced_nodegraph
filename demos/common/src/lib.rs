//! Shared utilities for iced_nodegraph demos.
//!
//! Provides screenshot capture and other common functionality.

mod content;
mod screenshot;

pub use content::{NodeContentStyle, simple_node};
pub use screenshot::{ScreenshotHelper, ScreenshotMessage};
