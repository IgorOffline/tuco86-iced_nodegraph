use std::f32::consts::FRAC_PI_2;

use iced::widget::{button, checkbox, column, container, pick_list, row, slider, text};
use iced::{Color, Element, Fill, Length, Rectangle, Size, Subscription, Theme};
use iced_sdf::{Curve, Drawable, Pattern, SdfPrimitive, Style, Tiling};

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("SDF Basic - iced_sdf")
        .theme(App::theme)
        .subscription(App::subscription)
        .antialiasing(true)
        .run()
}

// --- Shape definitions ---

struct Layer {
    drawable_idx: usize,
    style: Style,
}

struct ShapeEntry {
    name: &'static str,
    drawables: Vec<Drawable>,
    layers: Vec<Layer>,
    extent: f32,
}

fn build_static_entries() -> Vec<ShapeEntry> {
    let edge = Curve::bezier([-80.0, 30.0], [-30.0, -60.0], [30.0, 60.0], [80.0, -30.0]);
    let node = build_node_shape();
    vec![
        ShapeEntry {
            name: "Line (DF)",
            drawables: vec![Curve::line([-80.0, -40.0], [80.0, 40.0])],
            layers: vec![Layer { drawable_idx: 0, style: Style::distance_field() }],
            extent: 120.0,
        },
        ShapeEntry {
            name: "Point (DF)",
            drawables: vec![Curve::point([0.0, 0.0], FRAC_PI_2)],
            layers: vec![Layer { drawable_idx: 0, style: Style::distance_field() }],
            extent: 80.0,
        },
        ShapeEntry {
            name: "Arc (DF)",
            drawables: vec![Curve::arc_segment([0.0, 0.0], 50.0, -std::f32::consts::FRAC_PI_2, std::f32::consts::PI)],
            layers: vec![Layer { drawable_idx: 0, style: Style::distance_field() }],
            extent: 80.0,
        },
        ShapeEntry {
            name: "Bezier (DF)",
            drawables: vec![edge.clone()],
            layers: vec![Layer { drawable_idx: 0, style: Style::distance_field() }],
            extent: 120.0,
        },
        ShapeEntry {
            name: "Node (DF)",
            drawables: vec![build_node_shape()],
            layers: vec![Layer { drawable_idx: 0, style: Style::distance_field() }],
            extent: 140.0,
        },
        ShapeEntry {
            name: "Grid",
            drawables: vec![Tiling::grid(20.0, 20.0, 0.5)],
            layers: vec![Layer { drawable_idx: 0, style: Style::solid(Color::from_rgba(1.0, 1.0, 1.0, 0.15)) }],
            extent: 100.0,
        },
        ShapeEntry {
            name: "Dots",
            drawables: vec![Tiling::dots(15.0, 15.0, 2.0)],
            layers: vec![Layer { drawable_idx: 0, style: Style::solid(Color::from_rgba(0.4, 0.8, 1.0, 0.3)) }],
            extent: 60.0,
        },
        ShapeEntry {
            name: "Node (Styled)",
            drawables: vec![node],
            layers: vec![
                Layer { drawable_idx: 0, style: Style::stroke(Color::from_rgb(0.3, 0.3, 0.35), Pattern::solid(1.0)) },
                Layer { drawable_idx: 0, style: Style::solid(Color::from_rgb(0.14, 0.14, 0.16)) },
                Layer { drawable_idx: 0, style: Style::solid(Color::from_rgba(0.0, 0.0, 0.0, 0.3)).expand(4.0).blur(8.0) },
            ],
            extent: 140.0,
        },
        ShapeEntry {
            name: "Outline",
            drawables: vec![build_node_shape()],
            layers: vec![Layer {
                drawable_idx: 0,
                style: Style::solid(Color::from_rgb(0.14, 0.14, 0.16))
                    .outline(1.5, Color::from_rgb(0.4, 0.8, 1.0)),
            }],
            extent: 140.0,
        },
    ]
}

fn build_node_shape() -> Drawable {
    let w: f32 = 120.0;
    let h: f32 = 80.0;
    let cr: f32 = 8.0;
    let pr: f32 = 5.0;
    let left_pins: &[f32] = &[-25.0, 0.0, 25.0];
    let right_pins: &[f32] = &[-15.0, 15.0];

    let mut s = Curve::shape([-w / 2.0 + cr, -h / 2.0], FRAC_PI_2);
    s = s.line(w - 2.0 * cr);
    s = s.arc(cr, FRAC_PI_2);
    s = edge_with_pins(s, h - 2.0 * cr, right_pins, pr);
    s = s.arc(cr, FRAC_PI_2);
    s = s.line(w - 2.0 * cr);
    s = s.arc(cr, FRAC_PI_2);
    let left_reversed: Vec<f32> = left_pins.iter().rev().map(|&y| -y).collect();
    s = edge_with_pins(s, h - 2.0 * cr, &left_reversed, pr);
    s = s.arc(cr, FRAC_PI_2);
    s.close()
}

fn edge_with_pins(
    mut s: iced_sdf::ShapeBuilder, length: f32, pins: &[f32], pr: f32,
) -> iced_sdf::ShapeBuilder {
    let half = length / 2.0;
    let mut sorted: Vec<f32> = pins.iter().map(|&y| y + half).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut pos = 0.0;
    for &pin_pos in &sorted {
        let gap = pin_pos - pr - pos;
        if gap > 0.01 { s = s.line(gap); }
        s = s.angle(FRAC_PI_2)
             .arc(pr, -std::f32::consts::PI)
             .angle(FRAC_PI_2);
        pos = pin_pos + pr;
    }
    let remaining = length - pos;
    if remaining > 0.01 { s = s.line(remaining); }
    s
}

// --- Edge Editor State ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternKind {
    Solid, Dashed, Arrowed, Dotted, DashDotted, ArrowDotted,
}

impl PatternKind {
    const ALL: &[Self] = &[Self::Solid, Self::Dashed, Self::Arrowed, Self::Dotted, Self::DashDotted, Self::ArrowDotted];
}

impl std::fmt::Display for PatternKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Solid => "Solid",
            Self::Dashed => "Dashed",
            Self::Arrowed => "Arrowed",
            Self::Dotted => "Dotted",
            Self::DashDotted => "Dash-Dot",
            Self::ArrowDotted => "Arrow-Dot",
        })
    }
}

struct EdgeEditor {
    pattern: PatternKind,
    thickness: f32,
    dash: f32,
    gap: f32,
    flow_speed: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
}

impl Default for EdgeEditor {
    fn default() -> Self {
        Self {
            pattern: PatternKind::Solid,
            thickness: 3.0,
            dash: 12.0,
            gap: 6.0,
            flow_speed: 0.0,
            color_r: 0.2,
            color_g: 0.85,
            color_b: 1.0,
        }
    }
}

impl EdgeEditor {
    fn build_pattern(&self) -> Pattern {
        let p = match self.pattern {
            PatternKind::Solid => Pattern::solid(self.thickness),
            PatternKind::Dashed => Pattern::dashed(self.thickness, self.dash, self.gap),
            PatternKind::Arrowed => Pattern::arrowed(self.thickness, self.dash, self.gap),
            PatternKind::Dotted => Pattern::dotted(self.gap, self.thickness * 0.5),
            PatternKind::DashDotted => Pattern::dash_dotted(self.thickness, self.dash, self.gap, self.thickness * 0.4),
            PatternKind::ArrowDotted => Pattern::arrow_dotted(self.thickness, self.dash, self.gap, self.thickness * 0.4),
        };
        if self.flow_speed != 0.0 { p.flow(self.flow_speed) } else { p }
    }

    fn build_style(&self) -> Style {
        let color = Color::from_rgb(self.color_r, self.color_g, self.color_b);
        Style::stroke(color, self.build_pattern())
    }
}

// --- App ---

const EDGE_EDITOR_IDX: usize = 100;

struct App {
    selected: usize,
    debug_tiles: bool,
    time: f32,
    entries: Vec<ShapeEntry>,
    edge_editor: EdgeEditor,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected: 0,
            debug_tiles: false,
            time: 0.0,
            entries: build_static_entries(),
            edge_editor: EdgeEditor::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Select(usize),
    ToggleDebugTiles(bool),
    Tick,
    SetPattern(PatternKind),
    SetThickness(f32),
    SetDash(f32),
    SetGap(f32),
    SetFlow(f32),
    SetColorR(f32),
    SetColorG(f32),
    SetColorB(f32),
}

impl App {
    fn theme(&self) -> Theme { Theme::Dark }

    fn update(&mut self, message: Message) {
        match message {
            Message::Select(i) => self.selected = i,
            Message::ToggleDebugTiles(v) => self.debug_tiles = v,
            Message::Tick => self.time += 1.0 / 60.0,
            Message::SetPattern(p) => self.edge_editor.pattern = p,
            Message::SetThickness(v) => self.edge_editor.thickness = v,
            Message::SetDash(v) => self.edge_editor.dash = v,
            Message::SetGap(v) => self.edge_editor.gap = v,
            Message::SetFlow(v) => self.edge_editor.flow_speed = v,
            Message::SetColorR(v) => self.edge_editor.color_r = v,
            Message::SetColorG(v) => self.edge_editor.color_g = v,
            Message::SetColorB(v) => self.edge_editor.color_b = v,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let has_anim = if self.selected == EDGE_EDITOR_IDX {
            self.edge_editor.flow_speed != 0.0
        } else {
            self.entries.get(self.selected).is_some_and(|e| e.layers.iter().any(|l| l.style.is_animated()))
        };
        if has_anim {
            iced::window::frames().map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Sidebar
        let mut sidebar = column![].spacing(4).padding(8).width(160);
        for (i, entry) in self.entries.iter().enumerate() {
            sidebar = sidebar.push(
                button(text(entry.name).size(14)).width(Fill)
                    .on_press(Message::Select(i))
                    .style(if i == self.selected { button::primary } else { button::secondary })
            );
        }
        sidebar = sidebar.push(iced::widget::Space::new().height(4));
        sidebar = sidebar.push(
            button(text("Edge Editor").size(14)).width(Fill)
                .on_press(Message::Select(EDGE_EDITOR_IDX))
                .style(if self.selected == EDGE_EDITOR_IDX { button::primary } else { button::secondary })
        );
        sidebar = sidebar.push(iced::widget::Space::new().height(8));
        sidebar = sidebar.push(
            checkbox(self.debug_tiles).label("Debug Tiles")
                .on_toggle(Message::ToggleDebugTiles).size(14)
        );

        let sidebar = container(sidebar).height(Fill);

        // Main area
        let main_content: Element<'_, Message> = if self.selected == EDGE_EDITOR_IDX {
            self.view_edge_editor()
        } else if let Some(entry) = self.entries.get(self.selected) {
            let canvas = SdfCanvas {
                drawables: &entry.drawables,
                layers: &entry.layers,
                extent: entry.extent,
                debug_tiles: self.debug_tiles,
                time: self.time,
            };
            container(canvas).width(Fill).height(Fill).into()
        } else {
            text("No entry").into()
        };

        row![sidebar, main_content].into()
    }

    fn view_edge_editor(&self) -> Element<'_, Message> {
        let ed = &self.edge_editor;

        let controls = column![
            text("Pattern").size(13),
            pick_list(PatternKind::ALL, Some(ed.pattern), Message::SetPattern).width(140),
            text("Thickness").size(13),
            row![slider(0.5..=10.0, ed.thickness, Message::SetThickness).step(0.5), text(format!("{:.1}", ed.thickness)).size(12)].spacing(8),
            text("Dash/Segment").size(13),
            row![slider(2.0..=30.0, ed.dash, Message::SetDash).step(1.0), text(format!("{:.0}", ed.dash)).size(12)].spacing(8),
            text("Gap").size(13),
            row![slider(1.0..=20.0, ed.gap, Message::SetGap).step(1.0), text(format!("{:.0}", ed.gap)).size(12)].spacing(8),
            text("Flow Speed").size(13),
            row![slider(-100.0..=100.0, ed.flow_speed, Message::SetFlow).step(5.0), text(format!("{:.0}", ed.flow_speed)).size(12)].spacing(8),
            text("Color").size(13),
            row![text("R").size(12), slider(0.0..=1.0, ed.color_r, Message::SetColorR).step(0.05)].spacing(4),
            row![text("G").size(12), slider(0.0..=1.0, ed.color_g, Message::SetColorG).step(0.05)].spacing(4),
            row![text("B").size(12), slider(0.0..=1.0, ed.color_b, Message::SetColorB).step(0.05)].spacing(4),
        ].spacing(4).padding(8).width(200);

        let edge = Curve::bezier([-80.0, 30.0], [-30.0, -60.0], [30.0, 60.0], [80.0, -30.0]);
        let style = ed.build_style();

        let canvas = SdfCanvasOwned {
            drawables: vec![edge],
            styles: vec![style],
            extent: 120.0,
            debug_tiles: self.debug_tiles,
            time: self.time,
        };

        row![
            container(controls).height(Fill),
            container(canvas).width(Fill).height(Fill),
        ].into()
    }
}

// --- SdfCanvas widget ---

struct SdfCanvas<'a> {
    drawables: &'a [Drawable],
    layers: &'a [Layer],
    extent: f32,
    debug_tiles: bool,
    time: f32,
}

impl<'a, Message, Renderer> iced::advanced::Widget<Message, Theme, Renderer> for SdfCanvas<'a>
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer,
{
    fn size(&self) -> Size<Length> { Size::new(Length::Fill, Length::Fill) }

    fn layout(
        &mut self, _tree: &mut iced::advanced::widget::Tree, _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        iced::advanced::layout::Node::new(
            limits.width(Length::Fill).height(Length::Fill).resolve(Length::Fill, Length::Fill, Size::ZERO)
        )
    }

    fn draw(
        &self, _tree: &iced::advanced::widget::Tree, renderer: &mut Renderer,
        _theme: &Theme, _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>, _cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let viewport_min = bounds.width.min(bounds.height);
        let zoom = viewport_min * 0.333 / self.extent;
        let cam_x = bounds.width * 0.5 / zoom;
        let cam_y = bounds.height * 0.5 / zoom;
        let sb = [bounds.x, bounds.y, bounds.width, bounds.height];

        let mut prim = SdfPrimitive::new();
        for layer in self.layers {
            prim.push(&self.drawables[layer.drawable_idx], &layer.style, sb);
        }
        let prim = prim.camera(cam_x, cam_y, zoom).time(self.time).debug_tiles(self.debug_tiles);
        renderer.draw_primitive(bounds, prim);
    }
}

impl<'a, Message: 'a, Renderer> From<SdfCanvas<'a>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer + 'a,
{
    fn from(canvas: SdfCanvas<'a>) -> Self { Element::new(canvas) }
}

// --- SdfCanvasOwned: owns drawables + styles (for editors) ---

struct SdfCanvasOwned {
    drawables: Vec<Drawable>,
    styles: Vec<Style>,
    extent: f32,
    debug_tiles: bool,
    time: f32,
}

impl<Message, Renderer> iced::advanced::Widget<Message, Theme, Renderer> for SdfCanvasOwned
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer,
{
    fn size(&self) -> Size<Length> { Size::new(Length::Fill, Length::Fill) }

    fn layout(
        &mut self, _tree: &mut iced::advanced::widget::Tree, _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        iced::advanced::layout::Node::new(
            limits.width(Length::Fill).height(Length::Fill).resolve(Length::Fill, Length::Fill, Size::ZERO)
        )
    }

    fn draw(
        &self, _tree: &iced::advanced::widget::Tree, renderer: &mut Renderer,
        _theme: &Theme, _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>, _cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let viewport_min = bounds.width.min(bounds.height);
        let zoom = viewport_min * 0.333 / self.extent;
        let cam_x = bounds.width * 0.5 / zoom;
        let cam_y = bounds.height * 0.5 / zoom;
        let sb = [bounds.x, bounds.y, bounds.width, bounds.height];

        let mut prim = SdfPrimitive::new();
        for (i, style) in self.styles.iter().enumerate() {
            let drawable_idx = i.min(self.drawables.len() - 1);
            prim.push(&self.drawables[drawable_idx], style, sb);
        }
        let prim = prim.camera(cam_x, cam_y, zoom).time(self.time).debug_tiles(self.debug_tiles);
        renderer.draw_primitive(bounds, prim);
    }
}

impl<'a, Message: 'a, Renderer> From<SdfCanvasOwned> for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer + 'a,
{
    fn from(canvas: SdfCanvasOwned) -> Self { Element::new(canvas) }
}
