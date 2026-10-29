use eframe::egui::{
    self, Align, Color32, CornerRadius, CursorIcon, FontId, Layout, RichText, Sense, Stroke,
    StrokeKind, vec2,
};

use crate::codec::OutputFormat;
use crate::domain::{HISTORY_LIMIT, color_codes, hex_rgba};
use crate::i18n::{self, K};
use crate::theme as T;

use super::super::HunkApp;
use super::super::actions::AppAction;
use super::history::{HistorySwatch, paint_history_swatch};
use super::icons::{EyedropperButton, IconAction, eyedropper_button, icon_action_button};
use super::widgets::{HISTORY_COLUMNS, SWATCH_GAP, SWATCH_SIZE, hline, paint_checker};

impl HunkApp {
    pub(crate) fn show_color_panel(&mut self, ui: &mut egui::Ui) -> Vec<AppAction> {
        let mut actions = Vec::new();
        let icon_mode = self.format == OutputFormat::Ico;
        let sample = self.sampled_color;
        let sampled_position = self.sampled_position;
        let history = self.history.clone();
        let has_unlocked = history.iter().any(|entry| !entry.locked);

        T::soft_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(T::eyebrow(self.t(K::History)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let response = icon_action_button(ui, IconAction::Trash, has_unlocked)
                        .on_hover_text(self.t(K::ClearHistory));
                    if has_unlocked && response.clicked() {
                        actions.push(AppAction::ClearHistory);
                    }
                });
            });
            ui.add_space(4.0);

            let content_width = ui.available_width();
            let rows = HISTORY_LIMIT.div_ceil(HISTORY_COLUMNS);
            ui.spacing_mut().item_spacing = vec2(SWATCH_GAP, SWATCH_GAP);
            for row in 0..rows {
                let count = (HISTORY_LIMIT - row * HISTORY_COLUMNS).min(HISTORY_COLUMNS);
                let row_width =
                    count as f32 * SWATCH_SIZE + count.saturating_sub(1) as f32 * SWATCH_GAP;
                let offset = ((content_width - row_width) * 0.5).max(0.0);
                ui.horizontal(|ui| {
                    ui.add_space(offset);
                    for column in 0..count {
                        let index = row * HISTORY_COLUMNS + column;
                        let entry = history.get(index);
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(SWATCH_SIZE, SWATCH_SIZE), Sense::click());
                        paint_history_swatch(
                            ui.painter(),
                            rect,
                            HistorySwatch {
                                entry,
                                selected: sample,
                            },
                        );
                        if let Some(entry) = entry {
                            let response = response
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .on_hover_text(i18n::history_tooltip(
                                    self.language,
                                    &hex_rgba(entry.rgba),
                                    entry.locked,
                                ));
                            if response.clicked() {
                                self.sampled_color = Some(entry.rgba);
                                self.sampled_position = None;
                                self.eyedropper_active = false;
                                self.sample_pending_history = false;
                                actions.push(AppAction::RequestRepaint);
                            } else if response.clicked_by(egui::PointerButton::Secondary) {
                                actions.push(AppAction::ToggleHistoryLock(index));
                            }
                        }
                    }
                });
                if row + 1 < rows {
                    ui.add_space(SWATCH_GAP);
                }
            }
            ui.add_space(10.0);
            hline(ui);
            ui.add_space(10.0);

            ui.spacing_mut().interact_size.y = 20.0;
            ui.horizontal(|ui| {
                ui.label(T::eyebrow(self.t(K::Eyedropper)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let response = eyedropper_button(
                        ui,
                        EyedropperButton {
                            active: self.eyedropper_active,
                            enabled: !icon_mode,
                            label: self.t(K::Eyedropper),
                        },
                    );
                    let hint = if icon_mode {
                        self.t(K::EyedropperUnavailable)
                    } else if self.eyedropper_active {
                        self.t(K::DisableEyedropper)
                    } else {
                        self.t(K::EnableEyedropper)
                    };
                    let response = response.on_hover_text(hint);
                    if !icon_mode {
                        if response.drag_started() {
                            self.eyedropper_active = true;
                            actions.push(AppAction::RequestRepaint);
                        } else if response.clicked()
                            || (self.eyedropper_active
                                && response.clicked_by(egui::PointerButton::Secondary))
                        {
                            self.eyedropper_active = !self.eyedropper_active;
                            actions.push(AppAction::RequestRepaint);
                        }
                    }
                });
            });
            ui.add_space(3.0);

            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(36.0, 24.0), Sense::hover());
                let painter = ui.painter();
                paint_checker(painter, rect);
                if let Some(rgba) = sample {
                    painter.rect_filled(
                        rect,
                        CornerRadius::same(5),
                        Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]),
                    );
                }
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(5),
                    Stroke::new(1.0, T::BORDER_STRONG),
                    StrokeKind::Inside,
                );
                ui.add_space(2.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    let message = if icon_mode {
                        self.t(K::EyedropperIcoHint)
                    } else if self.eyedropper_active {
                        if sample.is_some() {
                            self.t(K::SampleCapturedChoose)
                        } else {
                            self.t(K::SampleClickOrDrag)
                        }
                    } else if sample.is_some() && sampled_position.is_none() {
                        self.t(K::SampleFromHistory)
                    } else if sample.is_some() {
                        self.t(K::SampleCaptured)
                    } else {
                        self.t(K::EyedropperIdle)
                    };
                    ui.add(
                        egui::Label::new(RichText::new(message).size(9.5).color(T::TEXT_MUTED))
                            .truncate(),
                    );
                    let coords = sampled_position
                        .map(|(x, y)| i18n::pixel_coords(self.language, x, y))
                        .unwrap_or_else(|| "\u{00A0}".to_owned());
                    ui.label(
                        RichText::new(coords)
                            .font(FontId::monospace(8.5))
                            .color(T::TEXT_FAINT),
                    );
                });
            });
            ui.add_space(2.0);

            ui.spacing_mut().interact_size.y = 14.0;
            ui.spacing_mut().item_spacing.y = 1.0;
            let codes = sample.map(color_codes).unwrap_or_else(|| {
                [
                    ("HEX", "—".to_owned()),
                    ("RGB", "—".to_owned()),
                    ("RGBA", "—".to_owned()),
                    ("HSL", "—".to_owned()),
                ]
            });
            for (label, value) in codes {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).size(9.0).color(T::TEXT_FAINT).strong());
                    ui.label(
                        RichText::new(&value)
                            .font(FontId::monospace(10.0))
                            .color(T::TEXT_MUTED),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let response = icon_action_button(ui, IconAction::Copy, sample.is_some())
                            .on_hover_text(i18n::copy_tooltip(self.language, label));
                        if sample.is_some() && response.clicked() {
                            actions.push(AppAction::CopyColor {
                                label: label.to_owned(),
                                text: value.clone(),
                            });
                        }
                    });
                });
            }
        });
        actions
    }
}
