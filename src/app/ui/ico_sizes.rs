use eframe::egui::{self, CornerRadius, RichText, Stroke, vec2};

use crate::codec::ICO_SIZES;
use crate::i18n::K;
use crate::theme as T;

use super::super::HunkApp;
use super::super::actions::AppAction;

impl HunkApp {
    pub(crate) fn show_ico_sizes(&mut self, ui: &mut egui::Ui, actions: &mut Vec<AppAction>) {
        ui.label(T::eyebrow(self.t(K::IconSizes)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (index, size) in ICO_SIZES.iter().enumerate() {
                let selected = self.ico_selected[index];
                let button = egui::Button::new(
                    RichText::new(size.to_string())
                        .size(10.5)
                        .color(if selected { T::ACCENT } else { T::TEXT_MUTED }),
                )
                .fill(if selected {
                    T::ACCENT_SOFT
                } else {
                    T::SURFACE_2
                })
                .stroke(Stroke::new(
                    1.0,
                    if selected { T::ACCENT } else { T::BORDER },
                ))
                .corner_radius(CornerRadius::same(8))
                .min_size(vec2(42.0, 26.0));
                if ui.add(button).clicked() {
                    let selected_count = self.ico_selected.iter().filter(|value| **value).count();
                    if selected && selected_count == 1 {
                        self.toast(self.t(K::SelectIcoSize).to_owned(), true, 3);
                    } else {
                        self.ico_selected[index] = !self.ico_selected[index];
                        actions.push(AppAction::RequestEncode(None));
                    }
                }
            }
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(self.t(K::IcoCentered))
                .size(9.5)
                .color(T::TEXT_FAINT),
        );
    }
}
