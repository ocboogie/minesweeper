use std::{f32::consts::PI, time::Instant};

use crate::canvas::Canvas;
use crate::load_image;
use crate::ms_frame::MinesweeperFrame;
use crate::{
    board::{Board, CellKind, CellState},
    board_ui::BoardUI,
};
use eframe::{
    egui::{Image, Sense, Ui, Widget},
    epaint::{vec2, Rect},
};
use egui::{
    emath::RectTransform, epaint::Shadow, include_image, pos2, style::Spacing, Align, Color32,
    Frame, ImageSource, InnerResponse, Layout, Margin, Pos2, Response, Shape, Stroke,
    TextureOptions, Vec2,
};

const DIGITS_IN_COUNTERS: usize = 3;
const FACE_SIZE: f32 = 24.0;

pub struct Minesweeper {
    pub board: BoardUI,
    pub canvas: Canvas,
    pub mines: usize,
    pub start: Instant,

    pub digits: [Image<'static>; 10],
    pub margin_corners: [Image<'static>; 2],
    pub faces: [Image<'static>; 5],
}

impl Minesweeper {
    fn load_digits() -> [Image<'static>; 10] {
        [
            load_image(include_image!("../assets/d-0.png")),
            load_image(include_image!("../assets/d-1.png")),
            load_image(include_image!("../assets/d-2.png")),
            load_image(include_image!("../assets/d-3.png")),
            load_image(include_image!("../assets/d-4.png")),
            load_image(include_image!("../assets/d-5.png")),
            load_image(include_image!("../assets/d-6.png")),
            load_image(include_image!("../assets/d-7.png")),
            load_image(include_image!("../assets/d-8.png")),
            load_image(include_image!("../assets/d-9.png")),
        ]
    }

    fn load_margin_corners() -> [Image<'static>; 2] {
        [
            load_image(include_image!("../assets/margin-corner-2.png")),
            load_image(include_image!("../assets/margin-corner-3.png")),
        ]
    }

    fn load_faces() -> [Image<'static>; 5] {
        [
            load_image(include_image!("../assets/face.png")),
            load_image(include_image!("../assets/face-pressed.png")),
            load_image(include_image!("../assets/face-pressing.png")),
            load_image(include_image!("../assets/face-won.png")),
            load_image(include_image!("../assets/face-lost.png")),
        ]
    }

    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Minesweeper {
            board: BoardUI::new(width, height, mines),
            mines,
            canvas: Canvas::new(),
            start: Instant::now(),
            digits: Self::load_digits(),
            margin_corners: Self::load_margin_corners(),
            faces: Self::load_faces(),
        }
    }

    // pub fn minesweeper_frame<R>(
    //     &self,
    //     ui: &mut Ui,
    //     margin: Margin,
    //     border: usize,
    //     protruded: bool,
    //     add_contents: impl FnOnce(&mut Ui) -> R,
    // ) -> InnerResponse<R> {
    // }

    pub fn counter(&self, ui: &mut Ui, number: usize) -> Response {
        MinesweeperFrame::new(1)
            .show(ui, |ui| {
                let response = ui
                    .horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(0.0);

                        for i in (0..DIGITS_IN_COUNTERS).rev() {
                            let digit: usize = (number / 10_usize.pow(i as u32)) % 10;
                            ui.add(self.digits[digit].clone());
                        }
                    })
                    .response;

                response
            })
            .inner
    }

    pub fn time_counter(&self, ui: &mut Ui) -> Response {
        let secs: usize = self.start.elapsed().as_secs().try_into().unwrap();
        self.counter(ui, secs)
    }

    pub fn mine_counter(&self, ui: &mut Ui) -> Response {
        let mines = self
            .board
            .board
            .cells
            .iter()
            .filter(|cell| cell.kind == CellKind::Mine && cell.state == CellState::Hidden)
            .count();
        self.counter(ui, mines)
    }

    pub fn face(&self, ui: &mut Ui) -> Response {
        MinesweeperFrame::new(1)
            .protruded()
            .show(ui, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::splat(FACE_SIZE), Sense::click());

                let face = if response.is_pointer_button_down_on() {
                    &self.faces[1]
                } else if self.board.win {
                    &self.faces[3]
                } else if self.board.game_over {
                    &self.faces[4]
                } else {
                    &self.faces[0]
                };

                face.paint_at(ui, rect);

                response
            })
            .inner
    }

    pub fn header(&mut self, ui: &mut Ui) -> Response {
        MinesweeperFrame::new(2)
            .margin(Margin::symmetric(7.0, 4.0))
            .show(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                    let space = ui.available_width();

                    self.time_counter(ui);

                    let counter_size = space - ui.available_width();

                    ui.add_space(((ui.available_width() - counter_size) / 2.0) - FACE_SIZE / 2.0);

                    if self.face(ui).clicked() {
                        self.board = BoardUI::new(
                            self.board.board.width,
                            self.board.board.height,
                            self.mines,
                        );
                        self.start = Instant::now();
                    }

                    ui.add_space(ui.available_width() - counter_size);

                    self.mine_counter(ui)
                })
                .inner
            })
            .inner
    }
}

impl Widget for &mut Minesweeper {
    fn ui(self, ui: &mut Ui) -> Response {
        // return self
        //     .canvas
        //     .show(ui, vec2(100.0, 50.0), |ui| {
        //         ui.button("Hello World");
        //     })
        //     .response;

        MinesweeperFrame::new(3)
            .floating()
            .margin(Margin::same(6.0))
            .show(ui, |ui| {
                self.header(ui);

                MinesweeperFrame::new(3)
                    .show(ui, |ui| {
                        // ui.add(&mut self.board)
                        self.canvas
                            .show(ui, self.board.size().into(), |ui| {
                                // ui.button("Hello World")
                                ui.add(&mut self.board)
                            })
                            .inner
                    })
                    .inner
            })
            .inner
    }
}

impl eframe::App for Minesweeper {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        ctx.set_pixels_per_point(4.0);
        ctx.tessellation_options_mut(|mut opts| {
            opts.feathering = false;
        });
        eframe::egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
}
