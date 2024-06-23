use egui::{emath::TSTransform, InnerResponse, LayerId, Margin, Order, Sense, Shape, Ui, Vec2};

use crate::ms_frame::MinesweeperFrame;

pub struct MinesweeperButton;

impl MinesweeperButton {
    pub fn new() -> Self {
        Self
    }

    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        MinesweeperFrame::new(1)
            .protruded()
            .show(ui, move |ui| {
                let frame_shape = ui.painter().add(Shape::Noop);
                let mut frame = MinesweeperFrame::new(2)
                    .sense(Sense::click())
                    .floating()
                    .margin(Margin::same(2.0));

                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);

                let layer_id = LayerId::new(Order::Foreground, id);

                let (res, content_ui) = ui
                    .with_layer_id(layer_id, |ui| frame.show_inner_contents(ui, add_contents))
                    .inner;

                if res.response.is_pointer_button_down_on() {
                    frame = frame.border(1).margin(Margin::same(3.0)).pressed();

                    ui.ctx().transform_layer_shapes(
                        layer_id,
                        TSTransform::from_translation(Vec2::splat(1.0)),
                    );
                }

                ui.painter()
                    .set(frame_shape, frame.paint(ui.ctx(), content_ui.min_rect()));

                res
            })
            .inner
    }
}
