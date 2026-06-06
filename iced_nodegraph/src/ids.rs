//! Generic ID types for user-defined node, pin, and edge identification.
//!
//! Nodes, pins, and edges carry the user's own id type directly; the library
//! never keeps a separate id-to-index map. These traits just collect the bounds
//! the widget needs on those id types.

use std::fmt::Debug;
use std::hash::Hash;

/// Trait for user-defined node identifiers.
///
/// Implement this trait on your own types to use them as node IDs:
/// ```rust
/// use iced_nodegraph::NodeId;
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// enum MyNodeId {
///     Input,
///     Process,
///     Output,
/// }
///
/// impl NodeId for MyNodeId {}
/// ```
pub trait NodeId: Clone + Eq + Hash + Debug + Send + Sync {}

/// Trait for user-defined pin identifiers.
///
/// Pins are identified within the context of a node, so you typically
/// use a per-node-type enum:
/// ```rust
/// use iced_nodegraph::PinId;
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// enum MathNodePins {
///     InputA,
///     InputB,
///     Output,
/// }
///
/// impl PinId for MathNodePins {}
/// ```
pub trait PinId: Clone + Eq + Hash + Debug + Send + Sync {}

/// Trait for user-defined edge identifiers.
///
/// Edges carry their own id (e.g. a database key), symmetric to nodes:
/// ```rust
/// use iced_nodegraph::EdgeId;
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// struct MyEdgeId(u64);
///
/// impl EdgeId for MyEdgeId {}
/// ```
pub trait EdgeId: Clone + Eq + Hash + Debug + Send + Sync {}

// Blanket implementations for common types

impl NodeId for usize {}
impl PinId for usize {}
impl EdgeId for usize {}

impl NodeId for u32 {}
impl PinId for u32 {}
impl EdgeId for u32 {}

impl NodeId for u64 {}
impl PinId for u64 {}
impl EdgeId for u64 {}

impl NodeId for String {}
impl PinId for String {}
impl EdgeId for String {}

impl NodeId for &'static str {}
impl PinId for &'static str {}
impl EdgeId for &'static str {}

// `()` is the default edge id: "this edge has no id". Nodes and pins always need
// a real id, so `()` implements only `EdgeId`.
impl EdgeId for () {}

// UUID support would require the uuid crate as a dependency
// Users can implement the traits for uuid::Uuid in their own code
