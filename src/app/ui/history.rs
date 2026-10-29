use eframe::egui::{Color32, CornerRadius, Rect, Stroke, StrokeKind, pos2, vec2};

use crate::domain::HistoryEntry;
use crate::theme as T;

use super::widgets::paint_checker;

pub(crate) struct HistorySwatch<'a> {
    pub entry: Option<&'a HistoryEntry>,
    pub selected: Option<[u8; 4]>,
}

pub(crate) fn paint_history_swatch(painter: &egui::Painter, rect: Rect, swatch: HistorySwatch<'_>) {
    let HistorySwatch { entry, selected } = swatch;
    let radius = CornerRadius::same(7);
    if let Some(entry) = entry {
        paint_checker(painter, rect);
        painter.rect_filled(
            rect,
            radius,
            Color32::from_rgba_unmultiplied(
                entry.rgba[0],
                entry.rgba[1],
                entry.rgba[2],
                entry.rgba[3],
            ),
        );
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(
                if entry.locked { 1.8 } else { 1.0 },
                if entry.locked {
                    T::ACCENT
                } else {
                    T::BORDER_STRONG
                },
            ),
            StrokeKind::Inside,
        );
        if selected == Some(entry.rgba) {
            painter.rect_stroke(
                rect.expand(2.0),
                CornerRadius::same(9),
                Stroke::new(2.0, T::ACCENT),
                StrokeKind::Outside,
            );
        }
        if entry.locked {
            paint_lock_badge(painter, rect);
        }
    } else {
        painter.rect(
            rect,
            radius,
            T::SURFACE_2,
            Stroke::new(1.0, T::BORDER),
            StrokeKind::Inside,
        );
    }
}

pub(crate) fn paint_lock_badge(painter: &egui::Painter, rect: Rect) {
    let center = rect.right_bottom() + vec2(-6.0, -6.0);
    painter.circle_filled(center, 6.0, Color32::from_black_alpha(190));
    let body = Rect::from_center_size(center + vec2(0.0, 1.0), vec2(5.5, 4.0));
    painter.rect_filled(body, CornerRadius::same(1), T::ACCENT);
    let shackle = Rect::from_center_size(center + vec2(0.0, -1.8), vec2(3.2, 3.2));
    painter.line_segment(
        [
            pos2(shackle.left(), shackle.center().y),
            pos2(shackle.left(), shackle.top()),
        ],
        Stroke::new(0.9, T::ACCENT),
    );
    painter.line_segment(
        [
            pos2(shackle.left(), shackle.top()),
            pos2(shackle.right(), shackle.top()),
        ],
        Stroke::new(0.9, T::ACCENT),
    );
    painter.line_segment(
        [
            pos2(shackle.right(), shackle.top()),
            pos2(shackle.right(), shackle.center().y),
        ],
        Stroke::new(0.9, T::ACCENT),
    );
}
