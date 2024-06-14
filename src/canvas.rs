use egui::{
    emath::{RectTransform, TSTransform},
    vec2, InnerResponse, Pos2, Rect, Ui, Vec2,
};

const MAX_ZOOM: f32 = 4.0;

pub struct Canvas {
    transform: TSTransform,
}

impl Canvas {
    pub fn new() -> Self {
        Canvas {
            transform: TSTransform::IDENTITY,
        }
    }

    pub fn adjusted_bounds(content_size: Vec2, screen_bounds: Rect) -> Rect {
        let screen_ratio = screen_bounds.aspect_ratio();
        let content_ratio = content_size.x / content_size.y;

        let resized = if screen_ratio > content_ratio {
            Vec2::splat(screen_bounds.size().y) * vec2(content_ratio, 1.0)
        } else {
            Vec2::splat(screen_bounds.size().x) * vec2(1.0, 1.0 / content_ratio)
        };

        Rect::from_min_size(
            screen_bounds.min
                + vec2(
                    (screen_bounds.size().x - resized.x).max(0.0) / 2.0,
                    (screen_bounds.size().y - resized.y).max(0.0) / 2.0,
                ),
            resized,
        )
    }

    pub fn adjusted_transform(&self, content_size: Vec2, screen_bounds: Rect) -> RectTransform {
        let adjusted_bounds = Self::adjusted_bounds(content_size, screen_bounds);

        let align_transform = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, content_size),
            adjusted_bounds,
        );

        RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, content_size),
            align_transform
                .transform_rect(self.transform * Rect::from_min_size(Pos2::ZERO, content_size)),
        )
    }

    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        content_size: Vec2,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let screen_bounds = ui.available_rect_before_wrap();

        // let adjusted_bounds = Rect::from_min_size(screen_bounds.min, resized);

        let response = ui.allocate_response(screen_bounds.size(), egui::Sense::drag());

        let transform = self.adjusted_transform(content_size, screen_bounds);

        if response.dragged() {
            self.transform.translation +=
                (response.drag_delta() / transform.scale()) * self.transform.scaling;
        }

        if let Some(pointer) = ui.ctx().input(|i| i.pointer.hover_pos()) {
            // Note: doesn't catch zooming / panning if a button in this PanZoom container is hovered.
            if response.hovered() {
                let pointer_in_layer = transform.inverse() * pointer;
                let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
                let pan_delta = ui.ctx().input(|i| i.raw_scroll_delta);

                // Zoom in on pointer:
                self.transform = self.transform
                    * TSTransform::from_translation(pointer_in_layer.to_vec2())
                    * TSTransform::from_scaling(zoom_delta)
                    * TSTransform::from_translation(-pointer_in_layer.to_vec2());

                // Limit scaling to avoid zooming out
                self.transform.scaling = self.transform.scaling.clamp(1.0, MAX_ZOOM);

                // Pan:
                self.transform.translation +=
                    (pan_delta / transform.scale().x) * self.transform.scaling;
            }
        }

        // Clamp view
        self.transform.translation = self.transform.translation.min(Vec2::splat(0.0));
        self.transform.translation = self
            .transform
            .translation
            .max(content_size * Vec2::splat(1.0 - self.transform.scaling));

        let transform = self.adjusted_transform(content_size, screen_bounds);
        let target_bounds = transform.transform_rect(Rect::from_min_size(Pos2::ZERO, content_size));

        let mut content_ui = ui.child_ui_with_id_source(target_bounds, *ui.layout(), response.id);

        content_ui.set_clip_rect(screen_bounds);

        let ret = add_contents(&mut content_ui);

        InnerResponse::new(ret, response)
    }
}
