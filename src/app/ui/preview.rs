use eframe::egui::{
    self, Align, CornerRadius, Frame, Layout, Margin, RichText, Sense, Stroke, Vec2, vec2,
};

use crate::codec::OutputFormat;
use crate::i18n::{self, K};
use crate::theme as T;

use super::super::HunkApp;
use super::super::MIN_ZOOM;
use super::super::actions::AppAction;
use super::canvas::CanvasContext;

impl HunkApp {
    pub(crate) fn show_preview(&mut self, root: &mut egui::Ui) -> Vec<AppAction> {
        let mut actions = Vec::new();
        let ctx = root.ctx().clone();
        egui::CentralPanel::default()
            .frame(Frame::new().fill(T::BG).inner_margin(Margin {
                left: 20,
                right: 30,
                top: 8,
                bottom: 12,
            }))
            .show(root, |ui| {
                egui::Panel::bottom("preview_footer_panel")
                    .resizable(false)
                    .show_separator_line(false)
                    .frame(Frame::new().inner_margin(Margin {
                        left: 0,
                        right: 0,
                        top: 8,
                        bottom: 0,
                    }))
                    .show(ui, |ui| {
                        self.show_preview_footer(ui);
                    });

                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.t(K::Preview)).size(17.0).color(T::TEXT));
                    ui.add_space(4.0);
                    if self.format != OutputFormat::Ico {
                        ui.add_space(2.0);
                        let zoom_percent = (self.zoom * 100.0).round() as u32;
                        if self.zoom > MIN_ZOOM + 0.001 {
                            let reset = egui::Button::new(
                                RichText::new(format!("Zoom {zoom_percent} %"))
                                    .size(9.5)
                                    .color(T::ACCENT),
                            )
                            .frame(false);
                            if ui.add(reset).on_hover_text(self.t(K::ResetZoom)).clicked() {
                                self.zoom = MIN_ZOOM;
                                self.pan = Vec2::ZERO;
                                ctx.request_repaint();
                            }
                        } else {
                            ui.label(RichText::new("Zoom 100 %").size(9.5).color(T::TEXT_FAINT));
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.busy() {
                            ui.add(egui::Spinner::new().size(12.0).color(T::ACCENT));
                            ui.label(
                                RichText::new(self.t(K::Processing))
                                    .size(9.5)
                                    .color(T::TEXT_MUTED),
                            );
                        } else if self.eyedropper_active {
                            ui.label(
                                RichText::new(self.t(K::EyedropperActiveHint))
                                    .size(9.5)
                                    .color(T::ACCENT),
                            );
                        } else if self.format == OutputFormat::Ico {
                            ui.label(
                                RichText::new(self.t(K::IcoGeneratedView))
                                    .size(9.5)
                                    .color(T::TEXT_FAINT),
                            );
                        } else if self.zoom > MIN_ZOOM + 0.001 {
                            ui.label(
                                RichText::new(self.t(K::PanZoomHint))
                                    .size(9.5)
                                    .color(T::TEXT_FAINT),
                            );
                        } else {
                            ui.label(
                                RichText::new(self.t(K::DividerZoomHint))
                                    .size(9.5)
                                    .color(T::TEXT_FAINT),
                            );
                        }
                    });
                });
                ui.add_space(7.0);

                let canvas_height = (ui.available_height() - 20.0).max(240.0);
                let frame = Frame::new()
                    .fill(T::BG_SOFT)
                    .stroke(Stroke::new(1.0, T::BORDER))
                    .corner_radius(CornerRadius::same(20))
                    .inner_margin(Margin::same(10));
                frame.show(ui, |ui| {
                    ui.set_min_size(vec2(ui.available_width(), canvas_height));
                    let size = vec2(ui.available_width(), canvas_height);
                    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
                    actions.extend(self.paint_canvas(CanvasContext {
                        ui,
                        rect,
                        response: &response,
                        ctx: &ctx,
                    }));
                });
            });
        actions
    }

    fn show_preview_footer(&mut self, ui: &mut egui::Ui) {
        let source_size = self.source.as_ref().map(|source| source.file_bytes);
        let output_size = self
            .result
            .as_ref()
            .filter(|result| result.format == self.format && !self.busy())
            .map(|result| result.size);

        ui.horizontal(|ui| {
            ui.label(T::eyebrow(self.t(K::Original)));
            ui.label(
                RichText::new(
                    source_size
                        .map(|bytes| i18n::localized_bytes(self.language, bytes))
                        .unwrap_or_else(|| "—".to_owned()),
                )
                .size(11.5)
                .color(T::TEXT),
            );
            T::bullet_separator(ui);
            ui.label(T::eyebrow(self.t(K::Result)));
            ui.label(
                RichText::new(
                    output_size
                        .map(|bytes| i18n::localized_bytes(self.language, bytes))
                        .unwrap_or_else(|| "—".to_owned()),
                )
                .size(11.5)
                .color(T::ACCENT),
            );
            if let (Some(original), Some(output)) = (source_size, output_size)
                && original > 0
            {
                let change = (1.0 - output as f64 / original as f64) * 100.0;
                let (text, color) = if change > 0.5 {
                    (format!("−{change:.0} %"), T::ACCENT)
                } else if change < -0.5 {
                    (format!("+{:.0} %", change.abs()), T::TEXT_MUTED)
                } else {
                    (self.t(K::EqualWeight).to_owned(), T::TEXT_MUTED)
                };
                ui.add_space(6.0);
                ui.label(RichText::new(text).size(9.5).color(color));
            }
        });
    }
}
