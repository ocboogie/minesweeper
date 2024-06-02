use egui::{
    emath::RectTransform, vec2, CursorIcon, Id, InnerResponse, Painter, PointerButton, Pos2, Rect,
    Response, Ui, Vec2,
};

pub struct Canvas {
    rect: Rect,

    last_click_pos_for_zoom: Option<Pos2>,
}

impl Canvas {
    pub fn new() -> Self {
        Canvas {
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(150.0)),

            last_click_pos_for_zoom: None,
        }
    }

    fn set_aspect_ratio(&mut self, aspect: f32) {
        let current_aspect = self.rect.aspect_ratio();

        let epsilon = 1e-5;
        if (current_aspect - aspect).abs() < epsilon {
            // Don't make any changes when the aspect is already almost correct.
            return;
        }

        if current_aspect < aspect {
            self.rect = self.rect.expand2(vec2(
                (aspect / current_aspect - 1.0) * self.rect.width() * 0.5,
                0.0,
            ));
        } else {
            self.rect = self.rect.expand2(vec2(
                0.0,
                (current_aspect / aspect - 1.0) * self.rect.height() * 0.5,
            ));
        }
    }

    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let rect_bounds = ui.available_rect_before_wrap();

        ui.painter()
            .rect_filled(rect_bounds, 0.0, egui::Color32::from_black_alpha(128));
        let mut response = ui.allocate_response(rect_bounds.size(), egui::Sense::drag());

        self.set_aspect_ratio(rect_bounds.aspect_ratio());

        let canvas_to_screen = RectTransform::from_to(self.rect, rect_bounds);

        // Draw pan
        if response.dragged_by(PointerButton::Primary) {
            response = response.on_hover_cursor(CursorIcon::Grabbing);
            let delta = -response.drag_delta();

            self.rect = self.rect.translate(delta / canvas_to_screen.scale().x);
        }

        // Box zoom
        if response.drag_started() && response.dragged_by(PointerButton::Middle) {
            self.last_click_pos_for_zoom = response.hover_pos();
        }

        let box_start_pos = self.last_click_pos_for_zoom;
        let box_end_pos = response.hover_pos();
        if let (Some(box_start_pos), Some(box_end_pos)) = (box_start_pos, box_end_pos) {
            response = response.on_hover_cursor(CursorIcon::ZoomIn);

            if response.drag_released() {
                self.rect = Rect::from_two_pos(box_start_pos, box_end_pos);

                self.last_click_pos_for_zoom = None;
            }
        }

        if let Some(hover_pos) = response.hover_pos() {
            let zoom_factor = ui.input(|i| i.zoom_delta());

            if zoom_factor != 1.0 {
                // let center = canvas_to_screen * hover_pos;

                self.rect =
                    Rect::from_center_size(self.rect.center(), self.rect.size() / zoom_factor);
            }

            let scroll_delta = ui.input(|i| i.raw_scroll_delta);
            if scroll_delta != Vec2::ZERO {
                self.rect = self
                    .rect
                    .translate(-scroll_delta / canvas_to_screen.scale().x);
            }
        }

        let mut content_ui =
            ui.child_ui(canvas_to_screen.transform_rect(rect_bounds), *ui.layout());

        content_ui.set_clip_rect(rect_bounds);

        let ret = add_contents(&mut content_ui);

        InnerResponse::new(ret, response)
    }
}
