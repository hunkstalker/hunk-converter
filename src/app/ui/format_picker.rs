use eframe::egui::{
    self, Align2, CornerRadius, CursorIcon, FontId, Sense, Stroke, StrokeKind, vec2,
};

use crate::codec::OutputFormat;
use crate::theme as T;

pub(super) struct FormatCard<'a> {
    pub format: OutputFormat,
    pub current: OutputFormat,
    pub width: f32,
    pub hint: &'a str,
}

pub(super) fn format_card(ui: &mut egui::Ui, card: FormatCard<'_>) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(card.width, 50.0), Sense::click());
    let selected = card.format == card.current;
    let hovered = response.hovered();
    let fill = if selected {
        T::ACCENT_SOFT
    } else if hovered {
        T::SURFACE_3
    } else {
        T::SURFACE_2
    };
    let stroke = Stroke::new(
        1.0,
        if selected {
            T::ACCENT
        } else if hovered {
            T::BORDER_STRONG
        } else {
            T::BORDER
        },
    );
    ui.painter().rect(
        rect,
        CornerRadius::same(12),
        fill,
        stroke,
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_top() + vec2(10.0, 10.0),
        Align2::LEFT_TOP,
        card.format.label(),
        FontId::proportional(13.5),
        T::TEXT,
    );
    ui.painter().text(
        rect.left_bottom() + vec2(10.0, -10.0),
        Align2::LEFT_BOTTOM,
        card.hint,
        FontId::proportional(9.5),
        if selected { T::ACCENT } else { T::TEXT_FAINT },
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}
