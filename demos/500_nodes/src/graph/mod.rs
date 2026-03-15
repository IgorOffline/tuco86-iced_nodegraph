mod layout;
mod procedural;

use iced_nodegraph::PinRef;

/// An edge connecting two pins.
type Edge = (PinRef<usize, usize>, PinRef<usize, usize>);

pub use procedural::generate_procedural_graph;
