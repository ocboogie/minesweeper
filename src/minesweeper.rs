use std::sync::atomic::Ordering;

use web_time::Instant;

use crate::canvas::Canvas;
use crate::ms_button::MinesweeperButton;
use crate::ms_frame::MinesweeperFrame;
use crate::ms_modal::MinesweeperModal;
use crate::solver::{generate_guessfree, solve_step};
use crate::utils::load_image;
use crate::{
    board::Board,
    minefield::{CellKind, CellState, Minefield},
};
use eframe::egui::{Image, Sense, Ui, Widget};
use egui::{include_image, Align, Color32, Frame, Label, Layout, Margin, Response, Vec2, Visuals};
use log::info;

const DIGITS_IN_COUNTERS: usize = 3;
const FACE_SIZE: f32 = 24.0;

struct GuessfreeGenerator {
    attempts: usize,
    start: usize,
    solving: bool,
}

pub struct Minesweeper {
    pub board: Board,
    pub canvas: Canvas,
    pub mines: usize,
    pub start: Instant,
    pub finished: Option<Instant>,
    pub started: bool,
    pub last_pressed: Option<(usize, usize, Instant)>,
    pub menu_open: bool,
    pub guessfree_generator: Option<GuessfreeGenerator>,
    // pub guessfree_generator: Option<GuessfreeGenerator>,
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

    fn with_size_and_mines(width: usize, height: usize, mines: usize) -> Self {
        Minesweeper {
            board: Board::from_minefield(Minefield::new(width, height)),
            mines,
            canvas: Canvas::new(),
            start: Instant::now(),
            finished: None,
            started: false,
            last_pressed: None,
            menu_open: false,
            guessfree_generator: None,
            digits: Self::load_digits(),
            margin_corners: Self::load_margin_corners(),
            faces: Self::load_faces(),
        }
    }

    pub fn new(ctx: &egui::Context) -> Self {
        Self::setup_fonts(ctx);
        let mut visuals = Visuals::light();
        visuals.override_text_color = Some(Color32::BLACK);

        ctx.set_visuals(visuals);

        Minesweeper::with_size_and_mines(9, 9, 10)
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
                        // self.reset();
                        self.menu_open = true;
                    }

                    ui.add_space(ui.available_width() - counter_size);

                    self.mine_counter(ui)
                })
                .inner
            })
            .inner
    }

    fn menu(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            if MinesweeperButton::new()
                .show(ui, |ui| {
                    ui.add(Label::new("Beginner").selectable(false));
                })
                .response
                .clicked()
            {
                self.board = Board::from_minefield(Minefield::new(9, 9));
                self.mines = 10;
                self.reset();
                self.menu_open = false;
            }
            if MinesweeperButton::new()
                .show(ui, |ui| {
                    ui.add(Label::new("Intermediate").selectable(false));
                })
                .response
                .clicked()
            {
                self.board = Board::from_minefield(Minefield::new(16, 16));
                self.mines = 40;
                self.reset();
                self.menu_open = false;
            }
            if MinesweeperButton::new()
                .show(ui, |ui| {
                    ui.add(Label::new("Expert").selectable(false));
                })
                .response
                .clicked()
            {
                self.board = Board::from_minefield(Minefield::new(30, 16));
                self.mines = 99;
                self.reset();
                self.menu_open = false;
            }
        })
        .inner
    }

    fn generator_status(ui: &mut Ui, generator: &GuessfreeGenerator) -> bool {
        ui.label(format!("Attempts: {}", generator.attempts));
        if MinesweeperButton::new()
            .show(ui, |ui| {
                ui.label("Cancel");
            })
            .response
            .clicked()
        {
            return true;
        }

        false
    }

    fn reset(&mut self) {
        self.canvas = Canvas::new();
        self.board = Board::from_minefield(Minefield::new(
            self.board.minefield.width,
            self.board.minefield.height,
        ));
        self.started = false;
        self.finished = None;
    }

    fn start_generating(&mut self, start: usize) {
        let mut minefield = Minefield::new(self.board.minefield.width, self.board.minefield.height);

        minefield.cells[start].state = CellState::Opened;

        self.board = Board::from_minefield(minefield);

        self.guessfree_generator = Some(GuessfreeGenerator {
            attempts: 0,
            start,
            solving: false,
        });
    }

    fn start(&mut self, minefield: Minefield) {
        self.board = Board::from_minefield(minefield);
        self.start = Instant::now();
        self.started = true;
        info!(
            "Started game with minefield: \n{}",
            self.board.minefield.format()
        );
    }

    fn setup_fonts(ctx: &egui::Context) {
        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "my_font".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/uni05_53.ttf")),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        // // Put my font as last fallback for monospace:
        // fonts
        //     .families
        //     .entry(egui::FontFamily::Monospace)
        //     .or_default()
        //     .push("my_font".to_owned());
        //
        // Tell egui to use these fonts:
        ctx.set_fonts(fonts);
    }

    pub fn generate_step(&mut self) {
        let width = self.board.minefield.width;
        let height = self.board.minefield.height;
        let mines = self.mines;

        if let Some(generator) = &mut self.guessfree_generator {
            let start = generator.start;

            if !generator.solving {
                loop {
                    generator.attempts += 1;

                    self.board.minefield = Minefield::generate(width, height, mines);

                    if self.board.minefield.cells[start].kind == CellKind::Mine {
                        continue;
                    }

                    self.board.minefield.open(start % width, start / width);
                    break;
                }
            }

            generator.solving = true;

            let minefield = self.board.minefield.clone();

            let mut new_minefield = solve_step(minefield.clone());

            if new_minefield == minefield {
                if new_minefield.is_solved() {
                    new_minefield.reset();
                    new_minefield.open(start % width, start / width);
                    self.start(new_minefield);
                    self.guessfree_generator = None;
                } else {
                    generator.solving = false;
                }
            } else {
                self.board.minefield = new_minefield;
            }
        }
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
                self.start_generating(y * self.board.minefield.width + x);
            }
        }

        self.last_pressed = self.board.pressed;

        if self.board.minefield.is_solved() && self.finished.is_none() {
            self.finished = Some(Instant::now());
        }

        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            self.board.minefield = solve_step(self.board.minefield.clone());
        }

        let mut menu_modal = MinesweeperModal::new(self.menu_open);

        menu_modal.show(ui, |ui| {
            self.menu(ui);
        });

        self.menu_open = menu_modal.open && self.menu_open;

        let mut cancel = false;

        if self.guessfree_generator.is_some() {
            self.generate_step();
        }

        if let Some(generator) = &mut self.guessfree_generator {
            MinesweeperModal::new(true).show(ui, |ui| {
                if Minesweeper::generator_status(ui, generator) {
                    cancel = true;
                }
            });
        }

        if cancel {
            self.guessfree_generator = None;
            self.board.minefield =
                Minefield::new(self.board.minefield.width, self.board.minefield.height);
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
        ctx.request_repaint();
        eframe::egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
}
