//! Theme Node
//!
//! Outputs the active theme's extended palette as color pins on the right, so
//! palette colors can be wired straight into config nodes' color inputs (or a
//! ColorQuad builder) instead of hand-picking colors. The values are resolved
//! from the current theme during propagation, so they follow theme changes.

use demo_common::NodeContentStyle;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Length};
use iced_nodegraph::pin;

use crate::nodes::{color_swatch, node_title_bar, pins};

/// Creates a Theme node exposing the extended palette as color outputs.
pub fn theme_node<'a, Message>(theme: &'a iced::Theme) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
{
    use pins::theme as t;

    let style = NodeContentStyle::output(theme);
    let p = theme.extended_palette();

    // One row per palette color: label on the left, the swatch carried by the
    // output pin on the right edge.
    let entry = |label: &'static str, id: &'static str, color: iced::Color| {
        row![
            text(label).size(10).width(Length::Fill),
            pin!(
                Right,
                id,
                color_swatch(Some(color)),
                Output,
                ::std::any::TypeId::of::<pins::ColorData>()
            ),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
    };

    let content = column![
        entry("background", t::BACKGROUND, p.background.base.color),
        entry("bg weak", t::BACKGROUND_WEAK, p.background.weak.color),
        entry("bg strong", t::BACKGROUND_STRONG, p.background.strong.color),
        entry("text", t::TEXT, p.background.base.text),
        entry("primary", t::PRIMARY, p.primary.base.color),
        entry("primary weak", t::PRIMARY_WEAK, p.primary.weak.color),
        entry("primary strong", t::PRIMARY_STRONG, p.primary.strong.color),
        entry("secondary", t::SECONDARY, p.secondary.base.color),
        entry("success", t::SUCCESS, p.success.base.color),
        entry("warning", t::WARNING, p.warning.base.color),
        entry("danger", t::DANGER, p.danger.base.color),
    ]
    .spacing(4);

    column![
        node_title_bar("Theme", style),
        container(content).padding([8, 10])
    ]
    .width(160.0)
    .into()
}
