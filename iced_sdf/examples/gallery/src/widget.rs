//! Custom widget that renders an SDF shape using iced_sdf.

use iced::widget::container;
use iced::{Color, Element, Fill, Length, Rectangle, Size, Theme};
use iced_sdf::{Layer, SdfPrimitive};

use crate::shapes::ShapeEntry;

/// IQ-style yellow for cursor visualization.
const CURSOR_COLOR: Color = Color {
    r: 1.0,
    g: 0.8,
    b: 0.0,
    a: 1.0,
};

/// Create an SDF canvas element that renders a shape entry.
///
/// If `layer_groups` is provided, each group is rendered as a separate
/// SdfPrimitive with its own debug_flags (for per-layer tile debug).
/// `extra_shapes` are additional shapes rendered with the same layer groups.
pub fn sdf_canvas<'a>(
    entry: &ShapeEntry,
    time: f32,
    layer_groups: Option<Vec<(Vec<Layer>, bool)>>,
    debug_tiles: bool,
    extra_shapes: &[iced_sdf::Sdf],
) -> Element<'a, crate::Message> {
    let shape = (entry.build)(time);

    let groups = match layer_groups {
        Some(g) => g,
        None => vec![((entry.layers)(), debug_tiles)],
    };

    let canvas = SdfCanvas {
        shape,
        extra_shapes: extra_shapes.to_vec(),
        layer_groups: groups,
        time,
        extent: entry.extent,
    };

    container(canvas)
        .width(Fill)
        .height(Fill)
        .center(Fill)
        .into()
}

/// Widget that renders SDF shapes with per-group debug control.
struct SdfCanvas {
    shape: iced_sdf::Sdf,
    extra_shapes: Vec<iced_sdf::Sdf>,
    /// Each group: (layers, debug_enabled). Rendered as separate SdfPrimitive.
    layer_groups: Vec<(Vec<Layer>, bool)>,
    time: f32,
    extent: f32,
}

impl<Message, Renderer> iced::advanced::Widget<Message, Theme, Renderer> for SdfCanvas
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let size = limits
            .width(Length::Fill)
            .height(Length::Fill)
            .resolve(Length::Fill, Length::Fill, Size::ZERO);
        iced::advanced::layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let center_x = bounds.x + bounds.width / 2.0;
        let center_y = bounds.y + bounds.height / 2.0;

        // Auto-zoom: fit shape extent into 2/3 of the smaller viewport dimension
        let viewport_min = bounds.width.min(bounds.height);
        let zoom = viewport_min * 0.333 / self.extent;

        let cam_x = center_x / zoom;
        let cam_y = center_y / zoom;

        // Render each layer group as a separate SdfPrimitive (for per-layer debug)
        let sb = [bounds.x, bounds.y, bounds.width, bounds.height];
        for (layers, debug) in &self.layer_groups {
            let mut prim = SdfPrimitive::new();
            // Primary shape
            prim.push(&self.shape, layers, sb);
            // Extra shapes (same layers, same bounds)
            for extra in &self.extra_shapes {
                prim.push(extra, layers, sb);
            }
            let prim = prim.camera(cam_x, cam_y, zoom)
                .time(self.time)
                .debug_tiles(*debug);
            renderer.draw_primitive(bounds, prim);
        }

        // Cursor distance overlay
        if let Some(pos) = cursor.position_over(bounds) {
            let cursor_world_x = (pos.x - center_x) / zoom;
            let cursor_world_y = (pos.y - center_y) / zoom;
            let cursor_world = glam::Vec2::new(cursor_world_x, cursor_world_y);

            let result = iced_sdf::evaluate(self.shape.node(), cursor_world);

            // Use the topmost layer's visual distance (stroke boundary, not raw SDF)
            let dist = self
                .layer_groups
                .last()
                .and_then(|(layers, _)| layers.last())
                .map(|l: &Layer| l.visual_distance(result.dist).abs())
                .unwrap_or(result.dist.abs());

            // Dot at cursor position (3px radius in screen space)
            let dot_radius = 3.0 / zoom;
            let dot = SdfPrimitive::single(
                iced_sdf::Sdf::circle([cursor_world_x, cursor_world_y], dot_radius),
            )
            .layers(vec![Layer::solid(CURSOR_COLOR)])
            .screen_bounds([bounds.x, bounds.y, bounds.width, bounds.height])
            .camera(cam_x, cam_y, zoom)
            .time(self.time);

            renderer.draw_primitive(bounds, dot);

            // Distance circle (radius = SDF distance, 1.5px outline)
            if dist > dot_radius * 2.0 {
                let outline_thickness = 1.5 / zoom;
                let circle = SdfPrimitive::single(
                    iced_sdf::Sdf::circle([cursor_world_x, cursor_world_y], dist)
                        .onion(outline_thickness),
                )
                .layers(vec![Layer::solid(CURSOR_COLOR)])
                .screen_bounds([bounds.x, bounds.y, bounds.width, bounds.height])
                .camera(cam_x, cam_y, zoom)
                .time(self.time);

                renderer.draw_primitive(bounds, circle);
            }
        }
    }
}

impl<'a, Message: 'a, Renderer> From<SdfCanvas> for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::Renderer + 'a,
{
    fn from(canvas: SdfCanvas) -> Self {
        Element::new(canvas)
    }
}
