use eframe::egui::{self, Align, Layout, Margin, RichText};

use crate::i18n::K;
use crate::theme as T;

use super::super::HunkApp;

impl HunkApp {
    pub(crate) fn show_footer(&self, ui: &mut egui::Ui) {
        egui::Panel::bottom("footer")
            .frame(
                eframe::egui::Frame::new()
                    .fill(T::BG)
                    .inner_margin(Margin::symmetric(30, 9)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "Rust + egui · Open Source · v{}",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .size(9.5)
                        .color(T::TEXT_FAINT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.hyperlink_to(
                            RichText::new("github.com/hunkstalker/hunk-converter")
                                .size(9.5)
                                .color(T::TEXT_MUTED),
                            "https://github.com/hunkstalker/hunk-converter",
                        )
                        .on_hover_text(self.t(K::OpenRepository));
                    });
                });
            });
    }
}
