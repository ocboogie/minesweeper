use crate::minefield::{CellKind, CellState, Minefield};
use eframe::{
    egui::{Image, Sense, Ui, Widget},
    epaint::{vec2, Rect},
};
use egui::{emath::RectTransform, pos2, Pos2, Response, TextureOptions, Vec2};

pub struct Board {
    pub minefield: Minefield,
    pub game_over: bool,
    pub win: bool,
    pub pressed: bool,
}

impl Board {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Board {
            minefield: Minefield::generate(width, height, mines),
            game_over: false,
            win: false,
            pressed: false,
        }
    }

    pub fn open_cell(&mut self, x: usize, y: usize) {
        if self.game_over {
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

        cell.state = CellState::Opened;

        if cell.kind == CellKind::Mine {
            self.game_over = true;
        }

        if mines != 0 {
            return;
        }

        for (x, y) in self.minefield.neighbors(x, y) {
            if self.minefield.cells[y * self.minefield.width + x].state == CellState::Hidden {
                self.open_cell(x, y);
            }
        }
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        if self.game_over {
            return;
        }
        let cell = &mut self.minefield.cells[y * self.minefield.width + x];
        if cell.state == CellState::Opened {
            return;
        }
        cell.state = match cell.state {
            CellState::Hidden => CellState::Flagged,
            CellState::Flagged => CellState::Hidden,
            CellState::Opened => unreachable!(),
        };
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
            _ => unreachable!(),
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

impl Widget for &mut Board {
    fn ui(self, ui: &mut Ui) -> Response {
        let screen_bounds = ui.available_rect_before_wrap();

        let board_to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, self.size().into()),
            screen_bounds,
        );

        let (_, mut response) =
            ui.allocate_exact_size(screen_bounds.size(), Sense::focusable_noninteractive());

        let mut pressed = None;

        for y in 0..self.minefield.height {
            for x in 0..self.minefield.width {
                let rect = board_to_screen.transform_rect(Rect::from_min_size(
                    pos2(x as f32 * 16.0, y as f32 * 16.0),
                    vec2(16.0, 16.0),
                ));

                let cell_response = ui.interact(rect, response.id.with((x, y)), Sense::click());

                if cell_response.is_pointer_button_down_on() {
                    if ui.input(|i| i.pointer.primary_down()) {
                        pressed = Some((x, y));
                    }
                }

                if cell_response.clicked() {
                    self.open_cell(x as usize, y as usize);
                } else if cell_response.secondary_clicked() || cell_response.long_touched() {
                    self.toggle_flag(x as usize, y as usize);
                }

                response = response.union(cell_response);
            }
        }

        self.pressed = pressed.is_some();

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

                    (CellState::Flagged, CellKind::Empty) if self.game_over => {
                        Board::incorrect_flag()
                    }
                    (CellState::Flagged, _) => Board::flag_cell(),

                    _ if pressed
                        .map(|(px, py)| {
                            (px == x && py == y)
                                || self.minefield.cells[py * self.minefield.width + px].state
                                    == CellState::Opened
                                    && x.abs_diff(px) <= 1
                                    && y.abs_diff(py) <= 1
                        })
                        .unwrap_or(false) =>
                    {
                        Board::empty_cell()
                    }
                    (CellState::Hidden, CellKind::Mine) if self.game_over => Board::revealed_mine(),
                    (CellState::Hidden, _) => Board::hidden_cell(),
                };

                image.paint_at(ui, rect);
            }
        }

        response
    }
}
