use web_time::Instant;

use crate::canvas::Canvas;
use crate::ms_frame::MinesweeperFrame;
use crate::solver::{generate_guessfree, solve_step};
use crate::utils::load_image;
use crate::{
    board::Board,
    minefield::{CellKind, CellState, Minefield},
};
use eframe::egui::{Image, Sense, Ui, Widget};
use egui::{include_image, Align, Frame, Layout, Margin, Response, Vec2};
use log::info;

const DIGITS_IN_COUNTERS: usize = 3;
const FACE_SIZE: f32 = 24.0;

pub struct Minesweeper {
    pub board: Board,
    pub canvas: Canvas,
    pub mines: usize,
    pub start: Instant,
    pub finished: Option<Instant>,
    pub started: bool,
    pub last_pressed: Option<(usize, usize, Instant)>,

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

    pub fn from_board(board: Board) -> Self {
        Minesweeper {
            mines: board.mines(),
            board,
            canvas: Canvas::new(),
            start: Instant::now(),
            finished: None,
            started: true,
            last_pressed: None,
            digits: Self::load_digits(),
            margin_corners: Self::load_margin_corners(),
            faces: Self::load_faces(),
        }
    }

    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Minesweeper {
            board: Board::from_minefield(Minefield::new(width, height)),
            mines,
            canvas: Canvas::new(),
            start: Instant::now(),
            finished: None,
            started: false,
            last_pressed: None,
            digits: Self::load_digits(),
            margin_corners: Self::load_margin_corners(),
            faces: Self::load_faces(),
        }
    }

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
        let secs: usize = match self.finished {
            _ if !self.started => 0,
            Some(finished) => finished
                .duration_since(self.start)
                .as_secs()
                .try_into()
                .unwrap(),
            None => self.start.elapsed().as_secs().try_into().unwrap(),
        };

        self.counter(ui, secs)
    }

    pub fn mine_counter(&self, ui: &mut Ui) -> Response {
        let mines = self
            .board
            .minefield
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
                } else if self.board.minefield.is_solved() {
                    &self.faces[3]
                } else if self.board.minefield.is_lost() {
                    &self.faces[4]
                } else if self.board.pressed.is_some() {
                    &self.faces[2]
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
                        self.reset();
                    }

                    ui.add_space(ui.available_width() - counter_size);

                    self.mine_counter(ui)
                })
                .inner
            })
            .inner
    }

    fn reset(&mut self) {
        self.board = Board::from_minefield(Minefield::new(
            self.board.minefield.width,
            self.board.minefield.height,
        ));
        // self.board = Board::new(self.board.minefield.width, self.board.minefield.width, 1);
        self.started = false;
        self.finished = None;
    }

    fn start(&mut self, start: usize) {
        self.board = Board::from_minefield(generate_guessfree(
            start,
            self.board.minefield.width,
            self.board.minefield.height,
            self.mines,
        ));
        self.start = Instant::now();
        self.started = true;

        info!(
            "Started game with minefield: \n{}",
            self.board.minefield.format()
        );
    }
}

impl Widget for &mut Minesweeper {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = MinesweeperFrame::new(3)
            .floating()
            .margin(Margin::same(6.0))
            .show(ui, |ui| {
                self.header(ui);

                MinesweeperFrame::new(3)
                    .show(ui, |ui| {
                        // ui.add(&mut self.board)
                        self.canvas
                            .show(ui, self.board.size().into(), |ui| ui.add(&mut self.board))
                            .inner
                    })
                    .inner
            })
            .inner;

        if !self.started && response.clicked() {
            if let Some((x, y, _)) = self.last_pressed {
                self.start(y * self.board.minefield.width + x);
            }
        }

        self.last_pressed = self.board.pressed;

        if self.board.minefield.is_solved() && self.finished.is_none() {
            self.finished = Some(Instant::now());
            dbg!();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            self.board.minefield = solve_step(self.board.minefield.clone())
        }

        response
    }
}

impl eframe::App for Minesweeper {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        ctx.set_pixels_per_point(4.0);
        ctx.tessellation_options_mut(|opts| {
            opts.feathering = false;
        });
        eframe::egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
}
