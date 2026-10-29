use std::time::Duration;

use eframe::egui::{self, Align, Layout, RichText};

use crate::i18n::K;
use crate::theme as T;

use super::super::ENCODE_DELAY_MS;
use super::super::HunkApp;
use super::super::actions::AppAction;

impl HunkApp {
    pub(crate) fn show_quality(&mut self, ui: &mut egui::Ui, actions: &mut Vec<AppAction>) {
        let lossy = self.format.is_lossy();
        let hint = if lossy {
            self.t(K::QualityHint)
        } else {
            self.t(K::LosslessHint)
        };
        let header = ui.horizontal(|ui| {
            ui.label(T::eyebrow(self.t(K::Quality)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let value = if lossy {
                    format!("{} %", self.quality)
                } else {
                    self.t(K::Lossless).to_owned()
                };
                ui.label(RichText::new(value).size(12.0).color(T::TEXT));
            });
        });
        header.response.on_hover_text(hint);

        if lossy {
            ui.add_space(3.0);
            ui.spacing_mut().slider_width = (ui.available_width() - 8.0).max(120.0);
            let slider = egui::Slider::new(&mut self.quality, 1..=100)
                .show_value(false)
                .trailing_fill(true);
            let response = ui.add(slider);
            response.clone().on_hover_text(hint);
            if response.changed() {
                actions.push(AppAction::RequestEncode(Some(Duration::from_millis(
                    ENCODE_DELAY_MS,
                ))));
            }
        }
    }
}
