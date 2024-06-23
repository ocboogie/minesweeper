use crate::utils::load_image;
use egui::{
    include_image, pos2, Color32, Context, Image, InnerResponse, Margin, Rect, Sense, Shape, Ui,
    Vec2,
};
use once_cell::sync::Lazy;

pub const SHADOW_COLOR: Color32 = Color32::from_rgb(128, 128, 128);
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
    Pressed,
}

pub struct MinesweeperFrame {
    margin: Margin,
    border: usize,
    kind: FrameKind,
    sense: Sense,
}

impl Default for MinesweeperFrame {
    fn default() -> Self {
        Self {
            margin: Margin::ZERO,
            border: 0,
            kind: FrameKind::Inscribed,
            sense: Sense::hover(),
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

    pub fn border(mut self, border: usize) -> Self {
        self.border = border;
        self
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

    pub fn pressed(mut self) -> Self {
        self.kind = FrameKind::Pressed;
        self
    }

    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    fn border_margin(&self) -> Margin {
        Margin::same(self.border as f32)
    }

    fn inner_rect(&self, content_rect: Rect) -> Rect {
        self.margin.expand_rect(content_rect)
    }

    fn outer_rect(&self, content_rect: Rect) -> Rect {
        self.border_margin()
            .expand_rect(self.inner_rect(content_rect))
    }

    pub fn paint(&self, ctx: &Context, content_rect: Rect) -> Shape {
        let b = self.border_margin();
        let inner_rect = self.inner_rect(content_rect);
        let outer_rect = self.outer_rect(content_rect);

        let mut shapes = Vec::new();

        shapes.push(Shape::rect_filled(inner_rect, 0.0, BACKGROUND_COLOR));
        // left
        shapes.push(Shape::rect_filled(
            Rect::from_min_max(
                outer_rect.min,
                pos2(
                    outer_rect.min.x + b.left,
                    outer_rect.max.y
                        - if self.kind == FrameKind::Pressed {
                            0.0
                        } else {
                            b.bottom
                        },
                ),
            ),
            0.0,
            if self.kind == FrameKind::Floating {
                HIGHTLIGHT_COLOR
            } else {
                SHADOW_COLOR
            },
        ));
        // top
        shapes.push(Shape::rect_filled(
            Rect::from_min_max(
                outer_rect.min,
                pos2(
                    outer_rect.max.x
                        - if self.kind == FrameKind::Pressed {
                            0.0
                        } else {
                            b.right
                        },
                    outer_rect.min.y + b.top,
                ),
            ),
            0.0,
            if self.kind == FrameKind::Floating {
                HIGHTLIGHT_COLOR
            } else {
                SHADOW_COLOR
            },
        ));
        // right
        if self.kind != FrameKind::Pressed {
            shapes.push(Shape::rect_filled(
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
            ));
            // bottom
            shapes.push(Shape::rect_filled(
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
            ));
        }

        if self.border > 1 && self.kind != FrameKind::Pressed {
            let corner = if self.border == 3 {
                MARGIN_CORNER_3
            } else if self.border == 2 {
                MARGIN_CORNER_2
            } else {
                panic!("Border too large: {:?}", self.border);
            };

            let uv = if self.kind == FrameKind::Floating {
                Rect::from_min_max(pos2(1.0, 1.0), pos2(0.0, 0.0))
            } else {
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
            };

            let corner = corner.load_for_size(ctx, Vec2::splat(16.0)).unwrap();

            shapes.push(Shape::image(
                corner.texture_id().unwrap(),
                Rect::from_min_max(
                    pos2(outer_rect.max.x - b.right, outer_rect.min.y),
                    pos2(outer_rect.max.x, outer_rect.min.y + b.top),
                ),
                uv,
                Color32::WHITE,
            ));

            shapes.push(Shape::image(
                corner.texture_id().unwrap(),
                Rect::from_min_max(
                    pos2(outer_rect.min.x, outer_rect.max.y - b.bottom),
                    pos2(outer_rect.min.x + b.left, outer_rect.max.y),
                ),
                uv,
                Color32::WHITE,
            ));
        }

        Shape::Vec(shapes)
    }

    pub fn show_inner_contents<R>(
        &self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> (InnerResponse<R>, Ui) {
        let b = Margin::same(self.border as f32);
        let outer_rect_bounds = ui.available_rect_before_wrap();
        let inner_rect_bounds = (b + self.margin).shrink_rect(outer_rect_bounds);

        let mut content_ui = ui.child_ui(inner_rect_bounds, *ui.layout());

        let ret = add_contents(&mut content_ui);

        let content_rect = content_ui.min_rect();

        let response = ui.allocate_rect(self.outer_rect(content_rect), self.sense);

        (InnerResponse::new(ret, response), content_ui)
    }

    pub fn show<R>(
        &self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let frame = ui.painter().add(Shape::Noop);

        let (ret, content_ui) = self.show_inner_contents(ui, add_contents);

        ui.painter()
            .set(frame, self.paint(ui.ctx(), content_ui.min_rect()));

        ret
    }
}
