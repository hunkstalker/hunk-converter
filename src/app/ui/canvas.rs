use std::sync::Arc;

use eframe::egui::{self, Align2, Color32, CursorIcon, FontId, Rect, Stroke, Vec2, pos2, vec2};

use crate::codec::OutputFormat;
use crate::i18n::{self, K};
use crate::theme as T;

use super::super::HunkApp;
use super::super::actions::AppAction;
use super::super::{MAX_ZOOM, MIN_ZOOM};
use super::widgets::{Badge, fit_rect, paint_badge, paint_checker};

pub(super) struct CanvasContext<'a> {
    pub ui: &'a egui::Ui,
    pub rect: Rect,
    pub response: &'a egui::Response,
    pub ctx: &'a egui::Context,
}

pub(super) struct ZoomInput {
    pub scroll_y: f32,
    pub pointer: egui::Pos2,
    pub fit_rect: Rect,
    pub area: Rect,
}

impl HunkApp {
    fn view_rect(&self, fit_rect: Rect, area: Rect) -> Rect {
        Rect::from_center_size(area.center() + self.pan, fit_rect.size() * self.zoom)
    }
    fn clamp_pan(&mut self, fit_rect: Rect, area: Rect) {
        if self.zoom <= MIN_ZOOM + f32::EPSILON {
            self.zoom = MIN_ZOOM;
            self.pan = Vec2::ZERO;
            return;
        }
        let size = fit_rect.size() * self.zoom;
        let max_x = ((size.x - area.width()) * 0.5).max(0.0);
        let max_y = ((size.y - area.height()) * 0.5).max(0.0);
        self.pan.x = self.pan.x.clamp(-max_x, max_x);
        self.pan.y = self.pan.y.clamp(-max_y, max_y);
    }
    fn zoom_at(&mut self, input: ZoomInput) {
        let ZoomInput {
            scroll_y,
            pointer,
            fit_rect,
            area,
        } = input;
        let factor = (scroll_y * 0.0025).exp();
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        if (new_zoom - self.zoom).abs() < f32::EPSILON {
            return;
        }

        let old_rect = self.view_rect(fit_rect, area);
        if old_rect.width() <= 0.0 || old_rect.height() <= 0.0 {
            return;
        }
        let normalized = vec2(
            ((pointer.x - old_rect.left()) / old_rect.width()).clamp(0.0, 1.0),
            ((pointer.y - old_rect.top()) / old_rect.height()).clamp(0.0, 1.0),
        );
        let new_size = fit_rect.size() * new_zoom;
        let new_center = pointer - normalized * new_size + new_size * 0.5;

        self.zoom = new_zoom;
        self.pan = new_center - area.center();
        self.clamp_pan(fit_rect, area);
    }
    pub(super) fn paint_canvas(&mut self, context: CanvasContext<'_>) -> Vec<AppAction> {
        let CanvasContext {
            ui,
            rect,
            response,
            ctx,
        } = context;
        let mut actions = Vec::new();
        let painter = ui.painter();
        let area = rect.shrink(12.0);
        let Some(source) = self.source.as_ref() else {
            return actions;
        };
        let source_texture = source.texture.id();
        let source_width = source.width;
        let source_height = source.height;
        let source_pixels = Arc::clone(&source.pixels);
        let source_size = vec2(source_width as f32, source_height as f32);
        let icon_mode = self.format == OutputFormat::Ico;

        if icon_mode {
            if let Some(result) = self
                .result
                .as_ref()
                .filter(|result| result.format == self.format)
            {
                let icon_size = vec2(result.width as f32, result.height as f32);
                let icon_rect = fit_rect(icon_size, area);
                paint_checker(painter, icon_rect);
                painter.image(
                    result.texture.id(),
                    icon_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                paint_badge(
                    painter,
                    Badge {
                        anchor: pos2(icon_rect.right() - 10.0, icon_rect.top() + 10.0),
                        align: Align2::RIGHT_TOP,
                        text: i18n::ico_badge(self.language, result.frame_count).as_str(),
                        background: T::ACCENT,
                        foreground: T::ACCENT_INK,
                    },
                );
            } else {
                painter.text(
                    area.center(),
                    Align2::CENTER_CENTER,
                    self.t(K::GeneratingIcon),
                    FontId::proportional(13.0),
                    T::TEXT_FAINT,
                );
            }
            return actions;
        }

        let fit_rect = fit_rect(source_size, area);
        self.clamp_pan(fit_rect, area);
        let mut image_rect = self.view_rect(fit_rect, area);

        let pointer = ctx.input(|input| input.pointer.hover_pos());
        let right_clicked =
            ctx.input(|input| input.pointer.button_pressed(egui::PointerButton::Secondary));
        let deactivated_eyedropper = self.eyedropper_active && right_clicked;
        if deactivated_eyedropper {
            self.eyedropper_active = false;
            self.sample_pending_history = false;
        }
        if response.hovered()
            && let Some(position) = pointer
            && image_rect.contains(position)
        {
            let scroll = ctx.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > f32::EPSILON {
                self.zoom_at(ZoomInput {
                    scroll_y: scroll,
                    pointer: position,
                    fit_rect,
                    area,
                });
                image_rect = self.view_rect(fit_rect, area);
            }
        }
        if !deactivated_eyedropper
            && response.dragged_by(egui::PointerButton::Secondary)
            && self.zoom > MIN_ZOOM
        {
            let delta = ctx.input(|input| input.pointer.delta());
            self.pan += delta;
            self.clamp_pan(fit_rect, area);
            image_rect = self.view_rect(fit_rect, area);
        }

        let view_rect = image_rect.intersect(area);
        let mut canvas = painter.clone();
        canvas.set_clip_rect(area);

        paint_checker(&canvas, image_rect);
        canvas.image(
            source_texture,
            image_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        let result_texture = self
            .result
            .as_ref()
            .filter(|result| result.format == self.format)
            .map(|result| result.texture.id());
        let split_x = image_rect.left() + image_rect.width() * self.split;
        let over_handle = result_texture.is_some()
            && pointer.is_some_and(|position| pointer_over_split(view_rect, split_x, position));

        if self.eyedropper_active {
            if response.hovered() {
                ctx.set_cursor_icon(CursorIcon::Crosshair);
            }
            if (ctx.input(|input| input.pointer.primary_down()) || response.clicked())
                && let Some(position) = pointer
                && image_rect.contains(position)
                && area.contains(position)
            {
                let nx = ((position.x - image_rect.left()) / image_rect.width()).clamp(0.0, 1.0);
                let ny = ((position.y - image_rect.top()) / image_rect.height()).clamp(0.0, 1.0);
                let x = (nx * source_width.saturating_sub(1) as f32).round() as u32;
                let y = (ny * source_height.saturating_sub(1) as f32).round() as u32;
                let pixel = source_pixels.get_pixel(x, y);
                self.sampled_color = Some(pixel.0);
                self.sampled_position = Some((x, y));
                self.sample_pending_history = true;
            }
        } else {
            if result_texture.is_some()
                && (response.dragged_by(egui::PointerButton::Primary) || response.clicked())
                && let Some(position) = ctx.input(|input| input.pointer.interact_pos())
            {
                self.split =
                    ((position.x - image_rect.left()) / image_rect.width()).clamp(0.015, 0.985);
            }
            if response.hovered() && over_handle {
                ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);
            } else if self.zoom > MIN_ZOOM
                && (response.hovered() || response.dragged_by(egui::PointerButton::Secondary))
            {
                ctx.set_cursor_icon(if response.dragged_by(egui::PointerButton::Secondary) {
                    CursorIcon::Grabbing
                } else {
                    CursorIcon::Grab
                });
            }
        }

        if self.sample_pending_history && ctx.input(|input| input.pointer.primary_released()) {
            if let Some(rgba) = self.sampled_color {
                actions.push(AppAction::RememberColor(rgba));
            }
            self.sample_pending_history = false;
        }

        if let Some(texture_id) = result_texture {
            let split_x = image_rect.left() + image_rect.width() * self.split;
            let split_clip_x = split_x.clamp(view_rect.left(), view_rect.right());
            let mut clipped = canvas.clone();
            clipped.set_clip_rect(Rect::from_min_max(
                view_rect.min,
                pos2(split_clip_x, view_rect.max.y),
            ));
            clipped.image(
                texture_id,
                image_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            if split_x >= view_rect.left()
                && split_x <= view_rect.right()
                && view_rect.height() > 24.0
            {
                let line_color = Color32::from_rgba_unmultiplied(255, 255, 255, 210);
                canvas.line_segment(
                    [
                        pos2(split_x, view_rect.top() + 8.0),
                        pos2(split_x, view_rect.bottom() - 8.0),
                    ],
                    Stroke::new(3.0, Color32::from_black_alpha(80)),
                );
                canvas.line_segment(
                    [
                        pos2(split_x, view_rect.top() + 8.0),
                        pos2(split_x, view_rect.bottom() - 8.0),
                    ],
                    Stroke::new(1.2, line_color),
                );
                let center = pos2(split_x, view_rect.center().y);
                canvas.circle_filled(center, 18.0, Color32::from_black_alpha(100));
                canvas.circle_filled(center, 16.0, T::BG_SOFT);
                canvas.circle_stroke(
                    center,
                    16.0,
                    Stroke::new(1.2, Color32::from_white_alpha(150)),
                );
                for direction in [-1.0_f32, 1.0] {
                    let tip = center + vec2(direction * 9.5, 0.0);
                    canvas.line_segment(
                        [center + vec2(direction * 4.0, -4.5), tip],
                        Stroke::new(1.4, T::TEXT),
                    );
                    canvas.line_segment(
                        [tip, center + vec2(direction * 4.0, 4.5)],
                        Stroke::new(1.4, T::TEXT),
                    );
                }
            }
        } else if self.busy() {
            canvas.text(
                view_rect.center(),
                Align2::CENTER_CENTER,
                self.t(K::PreparingResult),
                FontId::proportional(13.0),
                T::TEXT_FAINT,
            );
        }

        if self.eyedropper_active
            && let (Some(rgba), Some((sample_x, sample_y))) =
                (self.sampled_color, self.sampled_position)
        {
            let ratio_x = sample_x as f32 / source_width.saturating_sub(1).max(1) as f32;
            let ratio_y = sample_y as f32 / source_height.saturating_sub(1).max(1) as f32;
            let marker =
                image_rect.min + vec2(ratio_x * image_rect.width(), ratio_y * image_rect.height());
            canvas.circle_stroke(
                marker,
                8.0,
                Stroke::new(3.0, Color32::from_black_alpha(180)),
            );
            canvas.circle_stroke(marker, 8.0, Stroke::new(1.2, Color32::WHITE));
            canvas.circle_filled(
                marker,
                3.0,
                Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]),
            );
        }

        paint_badge(
            &canvas,
            Badge {
                anchor: view_rect.left_top() + vec2(10.0, 10.0),
                align: Align2::LEFT_TOP,
                text: self.format.badge(),
                background: T::ACCENT,
                foreground: T::ACCENT_INK,
            },
        );
        paint_badge(
            &canvas,
            Badge {
                anchor: view_rect.right_top() + vec2(-10.0, 10.0),
                align: Align2::RIGHT_TOP,
                text: self.t(K::Original),
                background: Color32::from_rgba_unmultiplied(10, 13, 16, 225),
                foreground: T::TEXT,
            },
        );
        actions
    }
}

pub(super) fn pointer_over_split(image_rect: Rect, split_x: f32, pointer: egui::Pos2) -> bool {
    let over_line = (pointer.x - split_x).abs() <= 10.0
        && pointer.y >= image_rect.top()
        && pointer.y <= image_rect.bottom();
    let over_circle = pointer.distance(pos2(split_x, image_rect.center().y)) <= 20.0;
    over_line || over_circle
}
