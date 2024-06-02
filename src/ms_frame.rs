use std::f32::consts::PI;

use crate::load_image;
use egui::{
    include_image, pos2, Color32, Image, InnerResponse, Margin, Rect, Sense, Shape, Ui, Vec2,
};
use once_cell::sync::Lazy;

const SHADOW_COLOR: Color32 = Color32::from_rgb(128, 128, 128);
const HIGHTLIGHT_COLOR: Color32 = Color32::WHITE;
const BACKGROUND_COLOR: Color32 = Color32::from_rgb(192, 192, 192);

const MARGIN_CORNER_2: Lazy<Image<'static>> =
    Lazy::new(|| load_image(include_image!("../assets/margin-corner-2.png")));
const MARGIN_CORNER_3: Lazy<Image<'static>> =
    Lazy::new(|| load_image(include_image!("../assets/margin-corner-3.png")));

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameKind {
    Protruded,
    Floating,
    Inscribed,
}

pub struct MinesweeperFrame {
    margin: Margin,
    border: usize,
    kind: FrameKind,
}

impl Default for MinesweeperFrame {
    fn default() -> Self {
        Self {
            margin: Margin::ZERO,
            border: 0,
            kind: FrameKind::Inscribed,
        }
    }
}

impl MinesweeperFrame {
    pub fn new(border: usize) -> Self {
        Self {
            border,
            ..Default::default()
        }
    }

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn protruded(mut self) -> Self {
        self.kind = FrameKind::Protruded;
        self
    }

    pub fn floating(mut self) -> Self {
        self.kind = FrameKind::Floating;
        self
    }

    pub fn show<R>(
        &self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let b = Margin::same(self.border as f32);
        let outer_rect_bounds = ui.available_rect_before_wrap();
        let inner_rect_bounds = (b + self.margin).shrink_rect(outer_rect_bounds);

        let mut content_ui = ui.child_ui(inner_rect_bounds, *ui.layout());

        let background = ui.painter().add(Shape::Noop);

        let ret = add_contents(&mut content_ui);

        let inner_rect = self.margin.expand_rect(content_ui.min_rect());
        let outer_rect = b.expand_rect(inner_rect);

        ui.painter().set(
            background,
            Shape::rect_filled(inner_rect, 0.0, BACKGROUND_COLOR),
        );

        // left
        ui.painter().rect_filled(
            Rect::from_min_max(
                outer_rect.min,
                pos2(outer_rect.min.x + b.left, outer_rect.max.y - b.bottom),
            ),
            0.0,
            if self.kind == FrameKind::Floating {
                HIGHTLIGHT_COLOR
            } else {
                SHADOW_COLOR
            },
        );
        // top
        ui.painter().rect_filled(
            Rect::from_min_max(
                outer_rect.min,
                pos2(outer_rect.max.x - b.right, outer_rect.min.y + b.top),
            ),
            0.0,
            if self.kind == FrameKind::Floating {
                HIGHTLIGHT_COLOR
            } else {
                SHADOW_COLOR
            },
        );
        // right
        ui.painter().rect_filled(
            Rect::from_min_max(
                pos2(outer_rect.max.x - b.right, outer_rect.min.y + b.top),
                pos2(outer_rect.max.x, outer_rect.max.y),
            ),
            0.0,
            if self.kind == FrameKind::Inscribed {
                HIGHTLIGHT_COLOR
            } else {
                SHADOW_COLOR
            },
        );
        // bottom
        ui.painter().rect_filled(
            Rect::from_min_max(
                pos2(outer_rect.min.x + b.left, outer_rect.max.y - b.bottom),
                pos2(outer_rect.max.x, outer_rect.max.y),
            ),
            0.0,
            if self.kind == FrameKind::Inscribed {
                HIGHTLIGHT_COLOR
            } else {
                SHADOW_COLOR
            },
        );

        if self.border > 1 {
            let mut corner = if self.border == 3 {
                MARGIN_CORNER_3
            } else if self.border == 2 {
                MARGIN_CORNER_2
            } else {
                panic!("Border too large: {:?}", self.border);
            };

            if self.kind == FrameKind::Floating {
                *corner = corner.clone().rotate(PI, Vec2::splat(0.5));
            }

            corner.paint_at(
                ui,
                Rect::from_min_max(
                    pos2(outer_rect.max.x - b.right, outer_rect.min.y),
                    pos2(outer_rect.max.x, outer_rect.min.y + b.top),
                ),
            );
            corner.paint_at(
                ui,
                Rect::from_min_max(
                    pos2(outer_rect.min.x, outer_rect.max.y - b.bottom),
                    pos2(outer_rect.min.x + b.left, outer_rect.max.y),
                ),
            );
        }

        let response = ui.allocate_rect(outer_rect, Sense::hover());

        InnerResponse::new(ret, response)
    }
}
