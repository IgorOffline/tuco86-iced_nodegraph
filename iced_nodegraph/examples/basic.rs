//! Minimal iced_nodegraph application.
//!
//! Two nodes wired together by a single edge. Drag a node by its title bar to
//! move it, drag from one pin to another to connect them, and click a connected
//! pin to unplug it.
//!
//! Run with:
//!
//!     cargo run -p iced_nodegraph --example basic

use iced::widget::{container, text};
use iced::{Element, Point, Theme, Vector};
use iced_nodegraph::prelude::*;
use iced_nodegraph::{edge, node, node_pin};

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("iced_nodegraph - basic")
        .theme(App::theme)
        .run()
}

/// A connection endpoint with the default `usize` node and pin ids.
type Pin = PinRef<usize, usize>;

struct App {
    /// Node positions in world space, indexed by node id. The graph reports
    /// drags through `on_move`; storing the result keeps nodes where they land.
    positions: Vec<Point>,
    /// Active connections between pins.
    edges: Vec<(Pin, Pin)>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            positions: vec![Point::new(120.0, 160.0), Point::new(440.0, 220.0)],
            edges: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Moved { id: usize, position: Point },
    // Box-select (drag on the background) or Ctrl+click multiple nodes, then
    // drag one of them: the graph reports the whole group shifted by `delta`.
    GroupMoved { ids: Vec<usize>, delta: Vector },
    Connected { from: Pin, to: Pin },
    Disconnected { from: Pin, to: Pin },
}

impl App {
    fn theme(&self) -> Theme {
        Theme::SolarizedLight
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Moved { id, position } => self.positions[id] = position,
            Message::GroupMoved { ids, delta } => {
                for id in ids {
                    self.positions[id] += delta;
                }
            }
            Message::Connected { from, to } => self.edges.push((from, to)),
            Message::Disconnected { from, to } => self.edges.retain(|&e| e != (from, to)),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = self.theme();

        let mut ng = node_graph()
            .on_move(|id, position| Message::Moved { id, position })
            .on_group_move(|ids, delta| Message::GroupMoved { ids, delta })
            .on_connect(|from, to| Message::Connected { from, to })
            .on_disconnect(|from, to| Message::Disconnected { from, to });

        // A source node carrying one output pin on its right edge. Pin ids are
        // `usize` here, so they must match the graph's default id type.
        // `simple_node` fills its width, so it needs a fixed-width parent.
        ng.push_node(node(
            0,
            self.positions[0],
            container(simple_node(
                "Source",
                NodeContentStyle::input(&theme),
                node_pin(PinSide::Right, 0usize, text("out")),
            ))
            .width(160.0),
        ));

        // A sink node with one input pin on its left edge.
        ng.push_node(node(
            1,
            self.positions[1],
            container(simple_node(
                "Sink",
                NodeContentStyle::output(&theme),
                node_pin(PinSide::Left, 0usize, text("in")),
            ))
            .width(160.0),
        ));

        // Re-create every stored connection each frame; the widget draws them
        // as bezier curves between the referenced pins.
        for &(from, to) in &self.edges {
            ng.push_edge(edge(from, to));
        }

        ng.into()
    }
}
