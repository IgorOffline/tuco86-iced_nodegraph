//! Configuration types for node graph styling.
//!
//! These types use `Option<T>` fields to allow partial overrides. Use `merge()`
//! to combine configs where `self` takes priority over `other`.

use iced::Color;
use iced_sdf::Pattern;

use super::{EdgeBorder, EdgeCurve, EdgeShadow, NodeBorder, NodeShadow, PinShape};

/// Partial node configuration for cascading style overrides.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeConfig {
    /// Fill color for the node body
    pub fill_color: Option<Color>,
    /// Corner radius for rounded corners
    pub corner_radius: Option<f32>,
    /// Node opacity (0.0 to 1.0)
    pub opacity: Option<f32>,
    /// Optional border (replaces any existing border)
    pub border: Option<NodeBorder>,
    /// Optional shadow (replaces any existing shadow)
    pub shadow: Option<ShadowConfig>,
}

impl NodeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill_color(mut self, color: impl Into<Color>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    /// Sets the border color (convenience: creates/modifies border).
    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        let c = color.into();
        let border = self.border.get_or_insert(NodeBorder::default());
        border.color = c;
        self
    }

    /// Sets the border width (convenience: creates/modifies border).
    pub fn border_width(mut self, width: f32) -> Self {
        let border = self.border.get_or_insert(NodeBorder::default());
        border.pattern = Pattern::solid(width);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    pub fn shadow(mut self, shadow: ShadowConfig) -> Self {
        self.shadow = Some(shadow);
        self
    }

    /// Removes the shadow (explicit override to no shadow).
    pub fn no_shadow(mut self) -> Self {
        self.shadow = Some(ShadowConfig::none());
        self
    }

    pub fn has_overrides(&self) -> bool {
        self.fill_color.is_some()
            || self.corner_radius.is_some()
            || self.opacity.is_some()
            || self.border.is_some()
            || self.shadow.is_some()
    }

    /// Merges two configs. Self takes priority, other fills gaps.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            fill_color: self.fill_color.or(other.fill_color),
            corner_radius: self.corner_radius.or(other.corner_radius),
            opacity: self.opacity.or(other.opacity),
            border: self.border.or(other.border),
            shadow: self.shadow.clone().or(other.shadow.clone()),
        }
    }
}

/// Shadow configuration for node drop shadows.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShadowConfig {
    /// Horizontal and vertical offset in world-space pixels
    pub offset: Option<(f32, f32)>,
    /// Blur radius in world-space pixels
    pub blur: Option<f32>,
    /// Shadow color
    pub color: Option<Color>,
    /// Whether shadow is enabled (false = explicit disable)
    pub enabled: Option<bool>,
}

impl ShadowConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a config that explicitly disables shadows.
    pub fn none() -> Self {
        Self {
            enabled: Some(false),
            ..Default::default()
        }
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Some((x, y));
        self
    }

    pub fn blur(mut self, blur: f32) -> Self {
        self.blur = Some(blur);
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            offset: self.offset.or(other.offset),
            blur: self.blur.or(other.blur),
            color: self.color.or(other.color),
            enabled: self.enabled.or(other.enabled),
        }
    }

    /// Resolves to a NodeShadow if enabled.
    pub fn resolve(&self) -> Option<NodeShadow> {
        if self.enabled == Some(false) {
            return None;
        }
        Some(NodeShadow {
            color: self.color.unwrap_or(Color::from_rgba(0.0, 0.0, 0.0, 0.3)),
            offset: self.offset.unwrap_or((4.0, 4.0)),
            blur: self.blur.unwrap_or(8.0),
        })
    }
}

/// Edge configuration for connection lines.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EdgeConfig {
    /// Color at source pin. TRANSPARENT = inherit from pin.
    pub start_color: Option<Color>,
    /// Color at target pin. TRANSPARENT = inherit from pin.
    pub end_color: Option<Color>,
    /// Stroke pattern (includes thickness, dash/gap, flow).
    pub pattern: Option<Pattern>,
    /// Optional outline on the stroke layer: (width, color).
    pub stroke_outline: Option<(f32, Color)>,
    /// Optional border ring around stroke.
    pub border: Option<EdgeBorder>,
    /// Optional shadow behind edge.
    pub shadow: Option<EdgeShadow>,
    /// Edge curve type.
    pub curve: Option<EdgeCurve>,
}

impl EdgeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a solid color (both start and end).
    pub fn solid_color(mut self, color: impl Into<Color>) -> Self {
        let c = color.into();
        self.start_color = Some(c);
        self.end_color = Some(c);
        self
    }

    pub fn start_color(mut self, color: impl Into<Color>) -> Self {
        self.start_color = Some(color.into());
        self
    }

    pub fn end_color(mut self, color: impl Into<Color>) -> Self {
        self.end_color = Some(color.into());
        self
    }

    pub fn pattern(mut self, pattern: Pattern) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Sets the stroke width (shorthand).
    pub fn width(mut self, width: f32) -> Self {
        let p = self.pattern.get_or_insert(Pattern::solid(width));
        p.thickness = width;
        self
    }

    /// Alias for width.
    pub fn thickness(self, thickness: f32) -> Self {
        self.width(thickness)
    }

    pub fn stroke_outline(mut self, width: f32, color: Color) -> Self {
        self.stroke_outline = Some((width, color));
        self
    }

    pub fn border(mut self, border: EdgeBorder) -> Self {
        self.border = Some(border);
        self
    }

    pub fn no_border(mut self) -> Self {
        self.border = None;
        self
    }

    pub fn shadow(mut self, shadow: EdgeShadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn no_shadow(mut self) -> Self {
        self.shadow = None;
        self
    }

    pub fn curve(mut self, curve: EdgeCurve) -> Self {
        self.curve = Some(curve);
        self
    }

    pub fn has_overrides(&self) -> bool {
        self.start_color.is_some()
            || self.end_color.is_some()
            || self.pattern.is_some()
            || self.stroke_outline.is_some()
            || self.border.is_some()
            || self.shadow.is_some()
            || self.curve.is_some()
    }

    /// Merges two edge configs. Self takes priority, other fills gaps.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start_color: self.start_color.or(other.start_color),
            end_color: self.end_color.or(other.end_color),
            pattern: self.pattern.or(other.pattern),
            stroke_outline: self.stroke_outline.or(other.stroke_outline),
            border: self.border.or(other.border),
            shadow: self.shadow.or(other.shadow),
            curve: self.curve.or(other.curve),
        }
    }
}

/// Pin configuration for connection points.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PinConfig {
    /// Pin indicator color
    pub color: Option<Color>,
    /// Pin indicator radius in world-space pixels
    pub radius: Option<f32>,
    /// Shape of the pin indicator
    pub shape: Option<PinShape>,
    /// Border color
    pub border_color: Option<Color>,
    /// Border width in world-space pixels
    pub border_width: Option<f32>,
}

impl PinConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn shape(mut self, shape: PinShape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = Some(width);
        self
    }

    pub fn has_overrides(&self) -> bool {
        self.color.is_some()
            || self.radius.is_some()
            || self.shape.is_some()
            || self.border_color.is_some()
            || self.border_width.is_some()
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            color: self.color.or(other.color),
            radius: self.radius.or(other.radius),
            shape: self.shape.or(other.shape),
            border_color: self.border_color.or(other.border_color),
            border_width: self.border_width.or(other.border_width),
        }
    }
}

/// Graph configuration for canvas and background.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GraphConfig {
    /// Background color of the canvas
    pub background_color: Option<Color>,
    /// Drag edge color when connection is invalid
    pub drag_edge_color: Option<Color>,
    /// Drag edge color when connection is valid
    pub drag_edge_valid_color: Option<Color>,
    /// Selection style configuration
    pub selection: Option<SelectionConfig>,
}

impl GraphConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background_color(mut self, color: impl Into<Color>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    pub fn drag_edge_color(mut self, color: impl Into<Color>) -> Self {
        self.drag_edge_color = Some(color.into());
        self
    }

    pub fn drag_edge_valid_color(mut self, color: impl Into<Color>) -> Self {
        self.drag_edge_valid_color = Some(color.into());
        self
    }

    pub fn selection(mut self, selection: SelectionConfig) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn has_overrides(&self) -> bool {
        self.background_color.is_some()
            || self.drag_edge_color.is_some()
            || self.drag_edge_valid_color.is_some()
            || self.selection.is_some()
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            background_color: self.background_color.or(other.background_color),
            drag_edge_color: self.drag_edge_color.or(other.drag_edge_color),
            drag_edge_valid_color: self.drag_edge_valid_color.or(other.drag_edge_valid_color),
            selection: match (&self.selection, &other.selection) {
                (Some(s), Some(o)) => Some(s.merge(o)),
                (Some(s), None) => Some(s.clone()),
                (None, Some(o)) => Some(o.clone()),
                (None, None) => None,
            },
        }
    }
}

/// Selection style configuration.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectionConfig {
    /// Border color for selected nodes
    pub border_color: Option<Color>,
    /// Border width for selected nodes
    pub border_width: Option<f32>,
    /// Fill color for box selection rectangle
    pub box_fill: Option<Color>,
    /// Border color for box selection rectangle
    pub box_border: Option<Color>,
}

impl SelectionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = Some(width);
        self
    }

    pub fn box_fill(mut self, color: impl Into<Color>) -> Self {
        self.box_fill = Some(color.into());
        self
    }

    pub fn box_border(mut self, color: impl Into<Color>) -> Self {
        self.box_border = Some(color.into());
        self
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            border_color: self.border_color.or(other.border_color),
            border_width: self.border_width.or(other.border_width),
            box_fill: self.box_fill.or(other.box_fill),
            box_border: self.box_border.or(other.box_border),
        }
    }
}

// Conversions from Style types to Config types

impl From<super::NodeStyle> for NodeConfig {
    fn from(style: super::NodeStyle) -> Self {
        Self {
            fill_color: Some(style.fill_color),
            corner_radius: Some(style.corner_radius),
            opacity: Some(style.opacity),
            border: style.border,
            shadow: style.shadow.map(|s| ShadowConfig {
                offset: Some(s.offset),
                blur: Some(s.blur),
                color: Some(s.color),
                enabled: Some(true),
            }),
        }
    }
}

impl From<super::EdgeStyle> for EdgeConfig {
    fn from(style: super::EdgeStyle) -> Self {
        Self {
            start_color: Some(style.start_color),
            end_color: Some(style.end_color),
            pattern: Some(style.pattern),
            stroke_outline: style.stroke_outline,
            border: style.border,
            shadow: style.shadow,
            curve: Some(style.curve),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_config_builder() {
        let config = NodeConfig::new()
            .fill_color(Color::from_rgb(0.5, 0.5, 0.5))
            .corner_radius(10.0)
            .opacity(0.9);

        assert_eq!(config.fill_color, Some(Color::from_rgb(0.5, 0.5, 0.5)));
        assert_eq!(config.corner_radius, Some(10.0));
        assert_eq!(config.opacity, Some(0.9));
        assert!(config.has_overrides());
    }

    #[test]
    fn test_empty_config_has_no_overrides() {
        let config = NodeConfig::new();
        assert!(!config.has_overrides());
    }

    #[test]
    fn test_edge_config_builder() {
        let config = EdgeConfig::new()
            .solid_color(Color::from_rgb(0.3, 0.6, 1.0))
            .thickness(3.0)
            .curve(EdgeCurve::Line);

        assert!(config.start_color.is_some());
        assert!(config.end_color.is_some());
        assert_eq!(config.curve, Some(EdgeCurve::Line));
    }

    #[test]
    fn test_shadow_config_none() {
        let config = ShadowConfig::none();
        assert_eq!(config.enabled, Some(false));
        assert!(config.resolve().is_none());
    }

    #[test]
    fn test_node_config_merge() {
        let defaults = NodeConfig::new().corner_radius(10.0).opacity(0.9);
        let specific = NodeConfig::new().fill_color(Color::from_rgb(1.0, 0.0, 0.0));
        let merged = specific.merge(&defaults);

        assert_eq!(merged.fill_color, Some(Color::from_rgb(1.0, 0.0, 0.0)));
        assert_eq!(merged.corner_radius, Some(10.0));
        assert_eq!(merged.opacity, Some(0.9));
    }

    #[test]
    fn test_edge_config_merge() {
        let defaults = EdgeConfig::new().thickness(2.0);
        let specific = EdgeConfig::new().solid_color(Color::WHITE);
        let merged = specific.merge(&defaults);

        assert_eq!(merged.start_color, Some(Color::WHITE));
    }

    #[test]
    fn test_pin_config_merge() {
        let defaults = PinConfig::new().radius(6.0).shape(PinShape::Circle);
        let specific = PinConfig::new().color(Color::BLACK);
        let merged = specific.merge(&defaults);

        assert_eq!(merged.color, Some(Color::BLACK));
        assert_eq!(merged.radius, Some(6.0));
        assert_eq!(merged.shape, Some(PinShape::Circle));
    }
}
