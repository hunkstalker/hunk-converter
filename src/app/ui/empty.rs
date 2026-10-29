use eframe::egui::{self, Align, CornerRadius, Frame, Layout, Margin, RichText, Stroke, vec2};

use crate::i18n::K;
use crate::theme as T;

use super::super::HunkApp;
use super::super::actions::AppAction;

impl HunkApp {
    pub(crate) fn show_empty(&mut self, root: &mut egui::Ui) -> Vec<AppAction> {
        let mut actions = Vec::new();
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(T::BG)
                    .inner_margin(Margin::symmetric(30, 18)),
            )
            .show(root, |ui| {
                let available = ui.available_size();
                let card_width = 680.0_f32.min(available.x - 20.0);
                let card_height = 500.0_f32.min(available.y - 20.0);
                let top_gap = ((available.y - card_height) / 2.0).max(0.0);
                ui.add_space(top_gap);
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    T::card_frame().show(ui, |ui| {
                        ui.set_width(card_width - 36.0);
                        ui.set_min_height(card_height - 36.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(12.0);
                            if let Some(icon) = &self.brand_icon {
                                ui.add(
                                    egui::Image::new((icon.id(), vec2(104.0, 104.0)))
                                        .corner_radius(CornerRadius::same(24)),
                                );
                            }
                            ui.add_space(14.0);
                            ui.label(
                                RichText::new(self.t(K::EmptyTitle))
                                    .size(34.0)
                                    .color(T::TEXT),
                            );
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(self.t(K::EmptySubtitle))
                                    .size(12.5)
                                    .color(T::TEXT_MUTED),
                            );
                            ui.add_space(22.0);
                            let choose = egui::Button::new(
                                RichText::new(self.t(K::SelectImage))
                                    .size(13.5)
                                    .color(T::ACCENT_INK)
                                    .strong(),
                            )
                            .fill(T::ACCENT)
                            .stroke(Stroke::new(1.0, T::ACCENT))
                            .corner_radius(CornerRadius::same(12))
                            .min_size(vec2(230.0, 44.0));
                            if ui.add(choose).clicked() {
                                actions.push(AppAction::OpenFileDialog);
                            }
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new("PNG · JPG · WEBP · BMP · TGA · SVG")
                                    .size(9.0)
                                    .color(T::TEXT_FAINT)
                                    .strong(),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(self.t(K::PrivacyNote))
                                    .size(9.5)
                                    .color(T::TEXT_FAINT),
                            );
                        });
                    });
                });
            });
        actions
    }
}
