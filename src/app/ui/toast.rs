use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Id, Sense, Stroke, StrokeKind, vec2,
};

use crate::theme as T;

use super::super::HunkApp;

impl HunkApp {
    pub(crate) fn show_toast(&mut self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else {
            return;
        };
        let now = Instant::now();
        if now >= toast.until {
            self.toast = None;
            return;
        }
        let remaining = toast.until - now;
        let text_color = if toast.error { T::DANGER } else { T::TEXT };
        let fill = if toast.error {
            Color32::from_rgb(42, 24, 26)
        } else {
            T::SURFACE_3
        };
        let stroke_color = if toast.error {
            Color32::from_rgba_unmultiplied(255, 126, 126, 140)
        } else {
            T::BORDER_STRONG
        };
        let message = toast.message.clone();

        egui::Area::new(Id::unique("toast"))
            .anchor(Align2::CENTER_TOP, vec2(0.0, 68.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                let galley =
                    ui.painter()
                        .layout(message, FontId::proportional(11.0), text_color, 420.0);
                let padding = vec2(16.0, 9.0);
                let (rect, _) =
                    ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::hover());
                let painter = ui.painter();
                painter.rect(
                    rect,
                    CornerRadius::same(10),
                    fill,
                    Stroke::new(1.0, stroke_color),
                    StrokeKind::Inside,
                );
                painter.galley(rect.min + padding, galley, text_color);
            });
        ctx.request_repaint_after(remaining.min(Duration::from_millis(100)));
    }
}
