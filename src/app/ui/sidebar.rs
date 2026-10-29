use eframe::egui::{self, Align, CornerRadius, Frame, Layout, Margin, RichText, Stroke, vec2};

use crate::codec::OutputFormat;
use crate::i18n::{self, K};
use crate::theme as T;

use super::super::HunkApp;
use super::super::actions::AppAction;
use super::format_picker::{FormatCard, format_card};
use super::widgets::small_text_button;

pub(super) struct FileCard<'a> {
    pub texture: egui::TextureId,
    pub name: &'a str,
    pub format: &'a str,
    pub file_bytes: u64,
    pub dimensions: (u32, u32),
}

impl HunkApp {
    pub(crate) fn show_sidebar(&mut self, root: &mut egui::Ui) -> Vec<AppAction> {
        let mut actions = Vec::new();
        let Some(source) = self.source.as_ref() else {
            return actions;
        };
        let texture = source.texture.id();
        let name = source.name.clone();
        let format = source.format.clone();
        let file_bytes = source.file_bytes;
        let dimensions = (source.width, source.height);
        let card = FileCard {
            texture,
            name: &name,
            format: &format,
            file_bytes,
            dimensions,
        };

        egui::Panel::left("settings_panel")
            .resizable(false)
            .exact_size(390.0)
            .show_separator_line(false)
            .frame(Frame::new().fill(T::BG).inner_margin(Margin {
                left: 30,
                right: 20,
                top: 14,
                bottom: 14,
            }))
            .show(root, |ui| {
                T::sidebar_frame().show(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    ui.set_width(ui.available_width());

                    let export_height = 68.0;
                    let scroll_height =
                        (ui.available_height() - self.color_panel_height - export_height - 8.0)
                            .max(160.0);
                    let mut settings_scroll = egui::ScrollArea::vertical()
                        .id_salt("sidebar_scroll")
                        .auto_shrink([false, false])
                        .max_height(scroll_height);
                    if scroll_height < 300.0 {
                        settings_scroll = settings_scroll.scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        );
                    }
                    settings_scroll.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.show_sidebar_top(ui, &card, &mut actions);
                    });

                    ui.add_space(4.0);
                    let mut color_actions = Vec::new();
                    let color_panel = ui.scope(|ui| {
                        color_actions = self.show_color_panel(ui);
                    });
                    actions.extend(color_actions);
                    self.color_panel_height = color_panel.response.rect.height();
                    let remaining = (ui.available_height() - export_height).max(0.0);
                    ui.add_space(remaining);
                    self.show_export(ui, &mut actions);
                });
            });
        actions
    }

    fn show_sidebar_top(
        &mut self,
        ui: &mut egui::Ui,
        card: &FileCard<'_>,
        actions: &mut Vec<AppAction>,
    ) {
        ui.horizontal(|ui| {
            ui.label(T::eyebrow(self.t(K::Image)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let response = small_text_button(ui, self.t(K::Change))
                    .on_hover_text(self.t(K::ChangeTooltip));
                if response.clicked() {
                    actions.push(AppAction::OpenFileDialog);
                }
            });
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new((card.texture, vec2(48.0, 48.0)))
                    .corner_radius(CornerRadius::same(9)),
            );
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.add_space(3.0);
                ui.label(RichText::new(card.name).size(12.5).color(T::TEXT).strong());
                ui.label(
                    RichText::new(format!(
                        "{} · {} × {} px · {}",
                        card.format,
                        card.dimensions.0,
                        card.dimensions.1,
                        i18n::localized_bytes(self.language, card.file_bytes)
                    ))
                    .size(9.5)
                    .color(T::TEXT_MUTED),
                );
            });
        });

        ui.add_space(6.0);
        super::widgets::hline(ui);
        ui.add_space(6.0);
        ui.label(T::eyebrow(self.t(K::OutputFormat)));
        ui.add_space(6.0);

        let gap = 6.0;
        let card_width = (ui.available_width() - gap * 2.0) / 3.0;
        let mut requested_format = None;
        for (row, formats) in OutputFormat::ALL.chunks(3).enumerate() {
            ui.horizontal(|ui| {
                for (column, format) in formats.iter().enumerate() {
                    let hint = i18n::format_hint(self.language, *format);
                    if format_card(
                        ui,
                        FormatCard {
                            format: *format,
                            current: self.format,
                            width: card_width,
                            hint,
                        },
                    )
                    .clicked()
                    {
                        requested_format = Some(*format);
                    }
                    if column + 1 < formats.len() {
                        ui.add_space(gap);
                    }
                }
            });
            if row + 1 < OutputFormat::ALL.len().div_ceil(3) {
                ui.add_space(gap);
            }
        }
        if let Some(format) = requested_format
            && self.set_format(format)
        {
            actions.push(AppAction::RequestEncode(None));
        }

        ui.add_space(10.0);
        if self.format == OutputFormat::Ico {
            self.show_ico_sizes(ui, actions);
        } else {
            self.show_quality(ui, actions);
        }
    }

    pub(crate) fn show_export(&mut self, ui: &mut egui::Ui, actions: &mut Vec<AppAction>) {
        let enabled = self.can_export();
        let label = i18n::export_label(self.language, self.format.extension());
        let button = egui::Button::new(
            RichText::new(&label)
                .size(13.0)
                .color(if enabled {
                    T::ACCENT_INK
                } else {
                    T::TEXT_FAINT
                })
                .strong(),
        )
        .fill(if enabled { T::ACCENT } else { T::SURFACE_3 })
        .stroke(Stroke::new(
            1.0,
            if enabled { T::ACCENT } else { T::BORDER },
        ))
        .corner_radius(CornerRadius::same(12))
        .min_size(vec2(ui.available_width(), 43.0));
        if ui.add(button).clicked() && enabled {
            actions.push(AppAction::Export);
        }
        ui.add_space(4.0);
        let hint = if self.busy() {
            self.t(K::Calculating)
        } else {
            self.t(K::ExportHint)
        };
        ui.label(RichText::new(hint).size(9.0).color(T::TEXT_FAINT));
    }
}
