use egui::{
    emath::RectTransform, vec2, CursorIcon, Id, InnerResponse, Painter, PointerButton, Pos2, Rect,
    Response, Ui, Vec2,
};

pub struct Canvas {
    canvas: Rect,

    last_click_pos_for_zoom: Option<Pos2>,
}

impl Canvas {
    pub fn new() -> Self {
        Canvas {
            canvas: Rect::from_min_size(Pos2::ZERO, vec2(1.0, 1.0)),

            last_click_pos_for_zoom: None,
        }
    }

    // fn set_aspect_ratio(&mut self, aspect: f32) {
    //     let current_aspect = self.rect.aspect_ratio();
    //
    //     let epsilon = 1e-5;
    //     if (current_aspect - aspect).abs() < epsilon {
    //         // Don't make any changes when the aspect is already almost correct.
    //         return;
    //     }
    //
    //     if current_aspect < aspect {
    //         self.rect = self.rect.expand2(vec2(
    //             (aspect / current_aspect - 1.0) * self.rect.width() * 0.5,
    //             0.0,
    //         ));
    //     } else {
    //         self.rect = self.rect.expand2(vec2(
    //             0.0,
    //             (current_aspect / aspect - 1.0) * self.rect.height() * 0.5,
    //         ));
    //     }
    // }

    pub fn adjusted_bounds(content_size: Vec2, screen_bounds: Rect) -> Rect {
        let screen_ratio = screen_bounds.aspect_ratio();
        let content_ratio = content_size.x / content_size.y;

        let resized = screen_bounds.size()
            * vec2(
                (1.0 / (screen_ratio * content_ratio)).min(1.0),
                (screen_ratio * content_ratio).min(1.0),
            );

        Rect::from_min_size(
            screen_bounds.min + vec2((screen_bounds.size().x - resized.x).max(0.0) / 2.0, 0.0),
            resized,
        )
    }

    pub fn transform(&self, content_size: Vec2, screen_bounds: Rect) -> RectTransform {
        let content_bounds = Rect::from_min_size(Pos2::ZERO, content_size);
        let adjusted_bounds = Self::adjusted_bounds(content_size, screen_bounds);

        let canvas_transform = RectTransform::from_to(
            // Rect::from_min_size(Pos2::ZERO, Vec2::splat(1.0)),
            self.canvas,
            content_bounds,
        );

        let align_transform = RectTransform::from_to(content_bounds, adjusted_bounds);

        RectTransform::from_to(
            self.canvas,
            align_transform.transform_rect(canvas_transform.transform_rect(self.canvas)),
        )
    }

    pub fn clamp_canvas(&mut self) {
        self.canvas = self
            .canvas
            .translate(-self.canvas.min.to_vec2().min(Vec2::splat(0.0)));
        self.canvas = self
            .canvas
            .translate((Vec2::splat(1.0) - self.canvas.max.to_vec2()).min(Vec2::splat(0.0)));
    }

    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        content_size: Vec2,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let screen_bounds = ui.available_rect_before_wrap();

        // let adjusted_bounds = Rect::from_min_size(screen_bounds.min, resized);

        let mut response = ui.allocate_response(screen_bounds.size(), egui::Sense::drag());

        let transform = self.transform(content_size, screen_bounds);

        // Draw pan
        if response.dragged_by(PointerButton::Primary) {
            response = response.on_hover_cursor(CursorIcon::Grabbing);
            let delta = -response.drag_delta();

            self.canvas = self.canvas.translate(delta / transform.scale().x);
        }

        // Box zoom
        // if response.drag_started() && response.dragged_by(PointerButton::Middle) {
        //     self.last_click_pos_for_zoom = response.hover_pos();
        // }
        //
        // let box_start_pos = self.last_click_pos_for_zoom;
        // let box_end_pos = response.hover_pos();
        // if let (Some(box_start_pos), Some(box_end_pos)) = (box_start_pos, box_end_pos) {
        //     response = response.on_hover_cursor(CursorIcon::ZoomIn);
        //
        //     if response.drag_released() {
        //         *rect = Rect::from_two_pos(box_start_pos, box_end_pos);
        //
        //         self.last_click_pos_for_zoom = None;
        //     }
        // }

        if let Some(hover_pos) = response.hover_pos() {
            let zoom_factor = ui.input(|i| i.zoom_delta());

            if zoom_factor != 1.0 {
                self.canvas = Rect::from_min_size(
                    self.canvas.min,
                    (self.canvas.size() / zoom_factor).clamp(Vec2::splat(0.1), Vec2::splat(1.0)),
                );
            }

            let scroll_delta = ui.input(|i| i.raw_scroll_delta);
            if scroll_delta != Vec2::ZERO {
                self.canvas = self.canvas.translate(-scroll_delta / transform.scale().x);
            }
        }

        self.clamp_canvas();

        let transform = self.transform(content_size, screen_bounds);
        let target_bounds =
            transform.transform_rect(Rect::from_min_size(Pos2::ZERO, Vec2::splat(1.0)));

        let mut content_ui = ui.child_ui_with_id_source(target_bounds, *ui.layout(), response.id);

        // content_ui.set_clip_rect(Rect::ZERO);
        content_ui.set_clip_rect(screen_bounds);

        let ret = add_contents(&mut content_ui);

        InnerResponse::new(ret, response)
    }
}
