//! Edge Configuration Node
//!
//! Builds an EdgeConfig from individual field inputs with inheritance support.

use iced::{
    Color, Length,
    alignment::Horizontal,
    widget::{column, container, row, text},
};
use iced_nodegraph::{
    EdgeBorder, EdgeConfig, EdgeCurve, EdgeShadow, NodeContentStyle, Pattern, pin,
};

use crate::nodes::{colors, node_title_bar, pins, section_header_with_pins};

/// Section expansion state for EdgeConfig nodes
#[derive(Debug, Clone, Default)]
pub struct EdgeSections {
    pub stroke: bool,
    pub pattern: bool,
    pub border: bool,
    pub shadow: bool,
}

impl EdgeSections {
    pub fn new_all_expanded() -> Self {
        Self {
            stroke: true,
            pattern: true,
            border: true,
            shadow: true,
        }
    }
}

/// Identifies which section to toggle in EdgeConfig
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSection {
    Stroke,
    Pattern,
    Border,
    Shadow,
}

/// Pattern type for simple selection (maps to iced_sdf::Pattern)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PatternType {
    #[default]
    Solid,
    Dashed,
    /// Arrow-like marks (///) crossing the edge
    Arrowed,
    /// Dashed with angled/parallelogram ends
    Angled,
    Dotted,
    DashDotted,
}

/// Collected inputs for EdgeConfigNode
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EdgeConfigInputs {
    /// Parent config to inherit from
    pub config_in: Option<EdgeConfig>,
    /// Individual field overrides
    pub start_color: Option<Color>,
    pub end_color: Option<Color>,
    pub thickness: Option<f32>,
    pub curve: Option<EdgeCurve>,
    /// Pattern settings
    pub pattern_type: Option<PatternType>,
    pub dash_length: Option<f32>,
    pub gap_length: Option<f32>,
    pub pattern_angle: Option<f32>,
    pub dot_radius: Option<f32>,
    /// Animation speed (0.0 = no animation, > 0.0 = animated)
    pub animation_speed: Option<f32>,
    /// Border settings
    pub border_width: Option<f32>,
    pub border_gap: Option<f32>,
    pub border_start_color: Option<Color>,
    pub border_end_color: Option<Color>,
    /// Shadow settings
    pub shadow_expand: Option<f32>,
    pub shadow_blur: Option<f32>,
    pub shadow_color: Option<Color>,
}

impl EdgeConfigInputs {
    /// Builds the final EdgeConfig by merging with parent
    pub fn build(&self) -> EdgeConfig {
        let parent = self.config_in.clone().unwrap_or_default();

        // Build pattern from inputs
        let pattern = self.build_pattern(&parent);

        // Colors
        let start_color = self.start_color.or(parent.start_color);
        let end_color = self.end_color.or(parent.end_color);

        // Build border config
        let has_border_overrides = self.border_width.is_some()
            || self.border_gap.is_some()
            || self.border_start_color.is_some()
            || self.border_end_color.is_some();

        let border = if has_border_overrides {
            let pb = parent.border.unwrap_or_default();
            Some(EdgeBorder {
                start_color: self.border_start_color.unwrap_or(pb.start_color),
                end_color: self.border_end_color.unwrap_or(pb.end_color),
                width: self.border_width.unwrap_or(pb.width),
                gap: self.border_gap.unwrap_or(pb.gap),
                outline: pb.outline,
            })
        } else {
            parent.border
        };

        // Build shadow config
        let has_shadow_overrides = self.shadow_blur.is_some()
            || self.shadow_expand.is_some()
            || self.shadow_color.is_some();

        let shadow = if has_shadow_overrides {
            let ps = parent.shadow.unwrap_or_default();
            Some(EdgeShadow {
                color: self.shadow_color.unwrap_or(ps.color),
                expand: self.shadow_expand.unwrap_or(ps.expand),
                blur: self.shadow_blur.unwrap_or(ps.blur),
            })
        } else {
            parent.shadow
        };

        EdgeConfig {
            start_color,
            end_color,
            pattern,
            border,
            shadow,
            curve: self.curve.or(parent.curve),
        }
    }

    /// Builds the Pattern from individual inputs
    fn build_pattern(&self, parent: &EdgeConfig) -> Option<Pattern> {
        let pattern_type = self.pattern_type.unwrap_or(PatternType::Solid);
        let thickness = self.thickness.unwrap_or(2.0);
        let dash = self.dash_length.unwrap_or(12.0);
        let gap = self.gap_length.unwrap_or(6.0);
        let angle = self.pattern_angle.unwrap_or(std::f32::consts::FRAC_PI_4);
        let dot_radius = self.dot_radius.unwrap_or(2.0);
        let speed = self.animation_speed.unwrap_or(0.0);

        let has_overrides = self.pattern_type.is_some() || self.thickness.is_some();

        if !has_overrides && self.pattern_type.is_none() {
            return parent.pattern;
        }

        let mut p = match pattern_type {
            PatternType::Solid => Pattern::solid(thickness),
            PatternType::Dashed => Pattern::dashed(thickness, dash, gap),
            PatternType::Arrowed => Pattern::arrowed(thickness, dash, gap, angle),
            PatternType::Angled => Pattern::dashed_angle(thickness, dash, gap, angle),
            PatternType::Dotted => Pattern::dotted(gap, dot_radius),
            PatternType::DashDotted => Pattern::dash_dotted(thickness, dash, gap, dot_radius),
        };

        if speed != 0.0 {
            p = p.flow(speed);
        }

        Some(p)
    }

    /// Returns the current pattern type
    pub fn get_pattern_type(&self) -> PatternType {
        self.pattern_type.unwrap_or(PatternType::Solid)
    }
}

/// Creates an EdgeConfig configuration node with all field inputs and collapsible sections
pub fn edge_config_node<'a, Message>(
    theme: &'a iced::Theme,
    inputs: &EdgeConfigInputs,
    sections: &EdgeSections,
    on_toggle: impl Fn(EdgeSection) -> Message + 'a,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
{
    let style = NodeContentStyle::process(theme);
    let result = inputs.build();

    // Config row: input left, typed output right
    let config_row = row![
        pin!(
            Left,
            pins::config::CONFIG,
            text("in").size(10),
            Input,
            pins::EdgeConfigData,
            colors::PIN_CONFIG
        ),
        container(text("")).width(Length::Fill),
        pin!(
            Right,
            pins::config::EDGE_OUT,
            text("out").size(10),
            Output,
            pins::EdgeConfigData,
            colors::PIN_CONFIG
        ),
    ]
    .align_y(iced::Alignment::Center);

    // Get values for display
    let start_color = result.start_color;
    let end_color = result.end_color;
    let thickness = result.pattern.map(|p| p.thickness);

    // Start color row
    let start_display: iced::Element<'a, Message> = if let Some(c) = start_color {
        container(text(""))
            .width(20)
            .height(12)
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(c)),
                border: iced::Border {
                    color: colors::PIN_ANY,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        text("--").size(9).into()
    };
    let start_row = row![
        pin!(
            Left,
            pins::config::START,
            text("start").size(10),
            Input,
            pins::ColorData,
            colors::PIN_COLOR
        ),
        container(start_display)
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // End color row
    let end_display: iced::Element<'a, Message> = if let Some(c) = end_color {
        container(text(""))
            .width(20)
            .height(12)
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(c)),
                border: iced::Border {
                    color: colors::PIN_ANY,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        text("--").size(9).into()
    };
    let end_row = row![
        pin!(
            Left,
            pins::config::END,
            text("end").size(10),
            Input,
            pins::ColorData,
            colors::PIN_COLOR
        ),
        container(end_display)
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Thickness row
    let thick_row = row![
        pin!(
            Left,
            pins::config::THICK,
            text("thick").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(text(thickness.map_or("--".to_string(), |v| format!("{:.1}", v))).size(9))
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Curve type row
    let curve_label = match result.curve {
        Some(EdgeCurve::BezierCubic) => "bezier",
        Some(EdgeCurve::Line) => "line",
        None => "--",
    };
    let curve_row = row![
        pin!(
            Left,
            pins::config::CURVE,
            text("curve").size(10),
            Input,
            pins::EdgeCurveData,
            colors::PIN_ANY
        ),
        container(text(curve_label).size(9))
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Pattern type row
    let pattern_label = match inputs.get_pattern_type() {
        PatternType::Solid => "solid",
        PatternType::Dashed => "dashed",
        PatternType::Arrowed => "arrowed",
        PatternType::Angled => "angled",
        PatternType::Dotted => "dotted",
        PatternType::DashDotted => "dash-dot",
    };
    let pattern_row = row![
        pin!(
            Left,
            pins::config::PATTERN,
            text("pattern").size(10),
            Input,
            pins::PatternTypeData,
            colors::PIN_ANY
        ),
        container(text(pattern_label).size(9))
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Dash length row
    let dash_row = row![
        pin!(
            Left,
            pins::config::DASH,
            text("dash").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(
            text(
                inputs
                    .dash_length
                    .map_or("--".to_string(), |v| format!("{:.1}", v))
            )
            .size(9)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Gap length row
    let gap_row = row![
        pin!(
            Left,
            pins::config::GAP,
            text("gap").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(
            text(
                inputs
                    .gap_length
                    .map_or("--".to_string(), |v| format!("{:.1}", v))
            )
            .size(9)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Pattern angle row
    let angle_display = inputs
        .pattern_angle
        .map_or("--".to_string(), |v| format!("{:.0} deg", v.to_degrees()));
    let angle_row = row![
        pin!(
            Left,
            pins::config::ANGLE,
            text("angle").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(text(angle_display).size(9))
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Animation speed row
    let speed_row = row![
        pin!(
            Left,
            pins::config::SPEED,
            text("speed").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(
            text(
                inputs
                    .animation_speed
                    .map_or("0".to_string(), |v| format!("{:.0}", v))
            )
            .size(9)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Border width row
    let border_width_row = row![
        pin!(
            Left,
            pins::config::BORDER_WIDTH,
            text("b.width").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(
            text(
                inputs
                    .border_width
                    .map_or("--".to_string(), |v| format!("{:.1}", v))
            )
            .size(9)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Border gap row
    let border_gap_row = row![
        pin!(
            Left,
            pins::config::BORDER_GAP,
            text("b.gap").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(
            text(
                inputs
                    .border_gap
                    .map_or("--".to_string(), |v| format!("{:.1}", v))
            )
            .size(9)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Border start color row
    let border_start_display: iced::Element<'a, Message> =
        if let Some(c) = inputs.border_start_color {
            container(text(""))
                .width(20)
                .height(12)
                .style(move |_: &_| container::Style {
                    background: Some(iced::Background::Color(c)),
                    border: iced::Border {
                        color: colors::PIN_ANY,
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            text("--").size(9).into()
        };
    let border_start_row = row![
        pin!(
            Left,
            pins::config::BORDER_START_COLOR,
            text("b.start").size(10),
            Input,
            pins::ColorData,
            colors::PIN_COLOR
        ),
        container(border_start_display)
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Border end color row
    let border_end_display: iced::Element<'a, Message> = if let Some(c) = inputs.border_end_color {
        container(text(""))
            .width(20)
            .height(12)
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(c)),
                border: iced::Border {
                    color: colors::PIN_ANY,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        text("--").size(9).into()
    };
    let border_end_row = row![
        pin!(
            Left,
            pins::config::BORDER_END_COLOR,
            text("b.end").size(10),
            Input,
            pins::ColorData,
            colors::PIN_COLOR
        ),
        container(border_end_display)
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Shadow blur row
    let shadow_blur_row = row![
        pin!(
            Left,
            pins::config::SHADOW_BLUR,
            text("s.blur").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(
            text(
                inputs
                    .shadow_blur
                    .map_or("--".to_string(), |v| format!("{:.1}", v))
            )
            .size(9)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Shadow expand row
    let shadow_expand_row = row![
        pin!(
            Left,
            pins::config::SHADOW_OFFSET,
            text("s.exp").size(10),
            Input,
            pins::Float,
            colors::PIN_NUMBER
        ),
        container(
            text(
                inputs
                    .shadow_expand
                    .map_or("--".to_string(), |v| format!("{:.1}", v))
            )
            .size(9)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Shadow color row
    let shadow_color_display: iced::Element<'a, Message> = if let Some(c) = inputs.shadow_color {
        container(text(""))
            .width(20)
            .height(12)
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(c)),
                border: iced::Border {
                    color: colors::PIN_ANY,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        text("--").size(9).into()
    };
    let shadow_color_row = row![
        pin!(
            Left,
            pins::config::SHADOW_COLOR,
            text("s.color").size(10),
            Input,
            pins::ColorData,
            colors::PIN_COLOR
        ),
        container(shadow_color_display)
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(iced::Alignment::Center);

    // Build content with collapsible sections
    let mut content_items: Vec<iced::Element<'_, Message>> = vec![config_row.into()];

    // Stroke section
    let stroke_collapsed_pins: Option<iced::Element<'_, Message>> = if !sections.stroke {
        Some(
            row![
                pin!(Left, pins::config::START, text("").size(1), Input, pins::ColorData, colors::PIN_COLOR).disable_interactions(),
                pin!(Left, pins::config::END, text("").size(1), Input, pins::ColorData, colors::PIN_COLOR).disable_interactions(),
                pin!(Left, pins::config::THICK, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
                pin!(Left, pins::config::CURVE, text("").size(1), Input, pins::EdgeCurveData, colors::PIN_ANY).disable_interactions(),
            ]
            .spacing(2)
            .into(),
        )
    } else {
        None
    };
    content_items.push(
        section_header_with_pins(
            "Stroke",
            sections.stroke,
            on_toggle(EdgeSection::Stroke),
            stroke_collapsed_pins,
        )
        .into(),
    );
    if sections.stroke {
        content_items.push(start_row.into());
        content_items.push(end_row.into());
        content_items.push(thick_row.into());
        content_items.push(curve_row.into());
    }

    // Pattern section
    let pattern_collapsed_pins: Option<iced::Element<'_, Message>> = if !sections.pattern {
        Some(
            row![
                pin!(Left, pins::config::PATTERN, text("").size(1), Input, pins::PatternTypeData, colors::PIN_ANY).disable_interactions(),
                pin!(Left, pins::config::DASH, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
                pin!(Left, pins::config::GAP, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
                pin!(Left, pins::config::ANGLE, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
                pin!(Left, pins::config::SPEED, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
            ]
            .spacing(2)
            .into(),
        )
    } else {
        None
    };
    content_items.push(
        section_header_with_pins(
            "Pattern",
            sections.pattern,
            on_toggle(EdgeSection::Pattern),
            pattern_collapsed_pins,
        )
        .into(),
    );
    if sections.pattern {
        content_items.push(pattern_row.into());
        content_items.push(dash_row.into());
        content_items.push(gap_row.into());
        content_items.push(angle_row.into());
        content_items.push(speed_row.into());
    }

    // Border section
    let border_collapsed_pins: Option<iced::Element<'_, Message>> = if !sections.border {
        Some(
            row![
                pin!(Left, pins::config::BORDER_WIDTH, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
                pin!(Left, pins::config::BORDER_GAP, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
                pin!(Left, pins::config::BORDER_START_COLOR, text("").size(1), Input, pins::ColorData, colors::PIN_COLOR).disable_interactions(),
                pin!(Left, pins::config::BORDER_END_COLOR, text("").size(1), Input, pins::ColorData, colors::PIN_COLOR).disable_interactions(),
            ]
            .spacing(2)
            .into(),
        )
    } else {
        None
    };
    content_items.push(
        section_header_with_pins(
            "Border",
            sections.border,
            on_toggle(EdgeSection::Border),
            border_collapsed_pins,
        )
        .into(),
    );
    if sections.border {
        content_items.push(border_width_row.into());
        content_items.push(border_gap_row.into());
        content_items.push(border_start_row.into());
        content_items.push(border_end_row.into());
    }

    // Shadow section
    let shadow_collapsed_pins: Option<iced::Element<'_, Message>> = if !sections.shadow {
        Some(
            row![
                pin!(Left, pins::config::SHADOW_BLUR, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
                pin!(Left, pins::config::SHADOW_OFFSET, text("").size(1), Input, pins::Float, colors::PIN_NUMBER).disable_interactions(),
                pin!(Left, pins::config::SHADOW_COLOR, text("").size(1), Input, pins::ColorData, colors::PIN_COLOR).disable_interactions(),
            ]
            .spacing(2)
            .into(),
        )
    } else {
        None
    };
    content_items.push(
        section_header_with_pins(
            "Shadow",
            sections.shadow,
            on_toggle(EdgeSection::Shadow),
            shadow_collapsed_pins,
        )
        .into(),
    );
    if sections.shadow {
        content_items.push(shadow_blur_row.into());
        content_items.push(shadow_expand_row.into());
        content_items.push(shadow_color_row.into());
    }

    let content = iced::widget::Column::with_children(content_items).spacing(4);

    column![
        node_title_bar("Edge Config", style),
        container(content).padding([8, 10])
    ]
    .width(150.0)
    .into()
}
