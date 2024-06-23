use egui::{Align2, Area, Frame, Margin, Pos2, Response, Rounding, Sense, Ui, Window};

use crate::ms_frame::MinesweeperFrame;

const OVERLAY_COLOR: egui::Color32 = egui::Color32::from_black_alpha(100);

pub struct MinesweeperModal {
    pub open: bool,
}

impl MinesweeperModal {
    pub fn new(open: bool) -> Self {
        Self { open }
    }

    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<Response> {
        if !self.open {
            return None;
        }

        let screen_rect = ui.ctx().input(|i| i.screen_rect);
        let area_resp = ui.allocate_rect(screen_rect, Sense::click());

        if area_resp.clicked() {
            self.open = false;
        }

        ui.painter()
            .rect_filled(screen_rect, Rounding::ZERO, OVERLAY_COLOR);

        let window = Window::new("")
            .id("modal_window".into())
            .frame(Frame::default())
            .open(&mut self.open)
            .title_bar(false)
            .vscroll(true)
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .resizable(false);

        window.show(&ui.ctx(), |ui| {
            MinesweeperFrame::new(3)
                .margin(Margin::same(10.0))
                .floating()
                .show(ui, add_contents)
        });

        Some(area_resp)
    }
}
