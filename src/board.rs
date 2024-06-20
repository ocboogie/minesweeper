use std::f32::consts::PI;

use crate::minefield::{CellKind, CellState, Minefield};
use eframe::{
    egui::{Image, Sense, Ui, Widget},
    epaint::{vec2, Rect},
};
use egui::{
    emath::{RectTransform, TSTransform},
    pos2, InputState, PointerButton, PointerState, Pos2, Response, TextureOptions, Vec2,
};
use web_time::Instant;

const FLAGGING_ANIMATION_DURATION: f32 = 0.10;
const FLAGGING_ANIMATION_SCALE: f32 = 1.3;

pub struct Board {
    pub minefield: Minefield,

    pub pressed: Option<(usize, usize, Instant)>,
    pub last_flag_toggle: Option<(usize, usize, Instant, bool)>,
}

impl Board {
    pub fn from_minefield(minefield: Minefield) -> Self {
        Board {
            minefield,
            pressed: None,
            last_flag_toggle: None,
        }
    }

    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Self::from_minefield(Minefield::generate(width, height, mines))
    }

    pub fn mines(&self) -> usize {
        self.minefield
            .cells
            .iter()
            .filter(|cell| cell.kind == CellKind::Mine)
            .count()
    }

    pub fn open_cell(&mut self, x: usize, y: usize) {
        if self.minefield.is_lost() || self.minefield.is_solved() {
            return;
        }

        let mines = self.minefield.count_mines(x, y);
        let cell = &mut self.minefield.cells[y * self.minefield.width + x];

        if cell.state == CellState::Opened {
            if mines == self.minefield.count_flags(x, y) {
                for (x, y) in self.minefield.neighbors(x, y) {
                    if self.minefield.cells[y * self.minefield.width + x].state == CellState::Hidden
                    {
                        self.open_cell(x, y);
                    }
                }
            }

            return;
        }

        if cell.kind == CellKind::Mine {
            cell.state = CellState::Opened;
        }

        self.minefield.open(x, y);
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        if self.minefield.is_lost() {
            return;
        }
        let cell = &mut self.minefield.cells[y * self.minefield.width + x];

        if cell.state == CellState::Opened {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let navigator = window.navigator();
            navigator.vibrate_with_duration(200);
        }

        match cell.state {
            CellState::Hidden => {
                self.last_flag_toggle = Some((x, y, Instant::now(), true));
                cell.state = CellState::Flagged;
            }
            CellState::Flagged => {
                self.last_flag_toggle = Some((x, y, Instant::now(), false));
                cell.state = CellState::Hidden;
            }
            CellState::Opened => unreachable!(),
        }
    }

    pub fn size(&self) -> (f32, f32) {
        (
            self.minefield.width as f32 * 16.0,
            self.minefield.height as f32 * 16.0,
        )
    }

    fn opened_cell(count: usize) -> Image<'static> {
        Self::pixelate(Image::new(match count {
            0 => return Board::empty_cell(),
            1 => egui::include_image!("../assets/1.png"),
            2 => egui::include_image!("../assets/2.png"),
            3 => egui::include_image!("../assets/3.png"),
            4 => egui::include_image!("../assets/4.png"),
            5 => egui::include_image!("../assets/5.png"),
            6 => egui::include_image!("../assets/6.png"),
            7 => egui::include_image!("../assets/7.png"),
            8 => egui::include_image!("../assets/8.png"),
            _ => {
                log::warn!("test2");
                unreachable!()
            }
        }))
    }

    fn pixelate(image: Image) -> Image {
        image.texture_options(TextureOptions::NEAREST)
    }

    fn empty_cell() -> Image<'static> {
        Self::pixelate(Image::new(egui::include_image!("../assets/empty.png")))
    }

    fn hidden_cell() -> Image<'static> {
        Self::pixelate(Image::new(egui::include_image!("../assets/hidden.png")))
    }

    fn flag_cell() -> Image<'static> {
        Self::pixelate(Image::new(egui::include_image!("../assets/flag.png")))
    }

    fn opened_mine() -> Image<'static> {
        Self::pixelate(Image::new(egui::include_image!(
            "../assets/opened_mine.png"
        )))
    }

    fn revealed_mine() -> Image<'static> {
        Self::pixelate(Image::new(egui::include_image!(
            "../assets/revealed_mine.png"
        )))
    }

    fn incorrect_flag() -> Image<'static> {
        Self::pixelate(Image::new(egui::include_image!(
            "../assets/incorrect_flag.png"
        )))
    }
}

const LONG_PRESS_DURATION: f64 = 0.25;

impl Widget for &mut Board {
    fn ui(self, ui: &mut Ui) -> Response {
        let screen_bounds = ui.available_rect_before_wrap();

        let board_to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, self.size().into()),
            screen_bounds,
        );

        let (_, response) = ui.allocate_exact_size(screen_bounds.size(), Sense::click());

        if let Some(pos) = response.interact_pointer_pos() {
            let pos = board_to_screen.inverse().transform_pos(pos);
            let x = pos.x as usize / 16;
            let y = pos.y as usize / 16;

            if ui.input(|input| input.pointer.button_pressed(PointerButton::Secondary)) {
                self.toggle_flag(x, y);
            }

            if let Some((px, py, time)) = self.pressed {
                if x == px && y == py {
                    if time.elapsed().as_secs_f64() >= LONG_PRESS_DURATION
                        && self.minefield.cells[y * self.minefield.width + x].state
                            != CellState::Opened
                    {
                        self.toggle_flag(x, y);
                        self.pressed = None;
                    } else if ui
                        .input(|input| input.pointer.button_released(PointerButton::Primary))
                    {
                        self.open_cell(x, y);
                        self.pressed = None;
                    }
                }
            } else if ui.input(|input| input.pointer.button_pressed(PointerButton::Primary)) {
                self.pressed = Some((x, y, Instant::now()));
            }
        }

        if ui.input(|input| !input.pointer.button_down(PointerButton::Primary)) {
            self.pressed = None;
        }

        let is_lost = self.minefield.is_lost();
        let is_solved = self.minefield.is_solved();

        for y in 0..self.minefield.height {
            for x in 0..self.minefield.width {
                let rect = board_to_screen.transform_rect(Rect::from_min_size(
                    pos2(x as f32 * 16.0, y as f32 * 16.0),
                    vec2(16.0, 16.0),
                ));

                let cell = &self.minefield.cells[y * self.minefield.width + x];

                let image = match (cell.state, cell.kind) {
                    (CellState::Opened, CellKind::Mine) => Board::opened_mine(),
                    (CellState::Opened, CellKind::Empty) => {
                        Board::opened_cell(self.minefield.count_mines(x, y))
                    }

                    (CellState::Flagged, CellKind::Empty) if is_lost => Board::incorrect_flag(),
                    (CellState::Flagged, _) => Board::flag_cell(),

                    _ if self.pressed.map_or(false, |(px, py, _)| {
                        (px == x && py == y)
                            || self.minefield.cells[py * self.minefield.width + px].state
                                == CellState::Opened
                                && x.abs_diff(px) <= 1
                                && y.abs_diff(py) <= 1
                    }) =>
                    {
                        Board::empty_cell()
                    }
                    (CellState::Hidden, CellKind::Mine) if is_lost => Board::revealed_mine(),
                    (CellState::Hidden, CellKind::Mine) if is_solved => Board::flag_cell(),
                    (CellState::Hidden, _) => Board::hidden_cell(),
                };

                image.paint_at(ui, rect);
            }
        }

        if let Some((x, y, time, flagging)) = self.last_flag_toggle {
            if time.elapsed().as_secs_f32() < FLAGGING_ANIMATION_DURATION {
                let rect = board_to_screen.transform_rect(Rect::from_min_size(
                    pos2(x as f32 * 16.0, y as f32 * 16.0),
                    vec2(16.0, 16.0),
                ));

                let alpha = time.elapsed().as_secs_f32() / FLAGGING_ANIMATION_DURATION;

                let mut alpha = ((alpha * PI) / 2.0).sin();

                if flagging {
                    alpha = 1.0 - alpha;
                }

                let rect = rect.expand(FLAGGING_ANIMATION_SCALE * 16.0 * alpha);

                Board::flag_cell().paint_at(ui, rect);
                ui.ctx().request_repaint();
            } else {
                self.last_flag_toggle = None;
            }
        }

        response
    }
}
