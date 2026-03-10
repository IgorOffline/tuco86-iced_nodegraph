//! GPU rendering effects for NodeGraph.
//!
//! - `GridPrimitive` - Background grid pattern (custom WGPU pipeline)
//!
//! Nodes, edges, pins, and overlays use `iced_sdf::SdfPrimitive` directly.

pub use primitives::{GridPrimitive, RenderContext};

pub(crate) mod pipeline;
pub mod primitives;
pub(crate) mod shared;
