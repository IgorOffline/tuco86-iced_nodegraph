//! Iced primitives for NodeGraph rendering.
//!
//! - `GridPrimitive` - Background grid pattern (custom WGPU pipeline)
//!
//! Nodes, edges, pins, and overlays use `iced_sdf::SdfPrimitive` directly.

use crate::node_graph::euclid::WorldPoint;

mod grid;

/// Shared per-frame rendering context for all primitives.
#[derive(Debug, Clone, Copy)]
pub struct RenderContext {
    pub camera_zoom: f32,
    pub camera_position: WorldPoint,
    pub time: f32,
}

pub use grid::GridPrimitive;
