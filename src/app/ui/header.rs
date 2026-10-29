use eframe::egui::{self, Align, Color32, CornerRadius, Layout, Margin, RichText, Stroke, vec2};

use crate::i18n::{K, Lang};
use crate::theme as T;

use super::super::HunkApp;
use super::super::actions::AppAction;

impl HunkApp {
    pub(crate) fn show_header(&mut self, ui: &mut egui::Ui) -> Vec<AppAction> {
        let mut actions = Vec::new();
        egui::Panel::top("app_header")
            .frame(
                eframe::egui::Frame::new()
                    .fill(T::BG)
                    .inner_margin(Margin::symmetric(30, 14)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(icon) = &self.brand_icon {
                        ui.add(
                            egui::Image::new((icon.id(), vec2(34.0, 34.0)))
                                .corner_radius(CornerRadius::same(8)),
                        );
                    }
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        ui.add_space(1.0);
                        ui.label(RichText::new("HUNK Converter").size(21.0).color(T::TEXT));
                        ui.label(
                            RichText::new("IMAGE TOOLBOX")
                                .size(8.5)
                                .color(T::TEXT_FAINT),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.source.is_some() {
                            let reset = egui::Button::new(
                                RichText::new(self.t(K::Reset))
                                    .size(11.5)
                                    .color(T::TEXT_MUTED),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, T::BORDER))
                            .corner_radius(CornerRadius::same(10))
                            .min_size(vec2(0.0, 32.0));
                            if ui.add(reset).clicked() {
                                actions.push(AppAction::Reset);
                            }
                        }

                        let mut language = self.language;
                        egui::ComboBox::from_id_salt("language_selector")
                            .selected_text(
                                RichText::new(language.short())
                                    .size(11.0)
                                    .color(T::TEXT_MUTED),
                            )
                            .width(64.0)
                            .show_ui(ui, |ui| {
                                for option in Lang::ALL {
                                    ui.selectable_value(&mut language, option, option.name());
                                }
                            })
                            .response
                            .on_hover_text(self.t(K::Language));
                        if language != self.language {
                            self.language = language;
                            actions.push(AppAction::PersistSettings);
                            actions.push(AppAction::RequestRepaint);
                        }
                    });
                });
            });
        actions
    }
}
