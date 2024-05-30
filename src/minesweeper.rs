use crate::board::{Board, CellKind, CellState};
use eframe::{
    egui::{Image, Sense, Ui, Widget},
    epaint::{vec2, Rect},
};
use egui::{emath::RectTransform, pos2, Pos2, Response, TextureOptions, Vec2};

pub struct Minesweeper {
    pub board: Board,
    pub started: bool,
    pub game_over: bool,
    pub win: bool,
}

impl Minesweeper {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Minesweeper {
            board: Board::generate(width, height, mines),
            started: false,
            game_over: false,
            win: false,
        }
    }

    pub fn open_cell(&mut self, x: usize, y: usize) {
        if self.game_over {
            return;
        }

        if !self.started {
            self.started = true;
        }

        let mines = self.board.count_mines(x, y);
        let cell = &mut self.board.cells[y * self.board.width + x];

        if cell.state == CellState::Opened {
            if mines == self.board.count_flags(x, y) {
                for (x, y) in self.board.neighbors(x, y) {
                    if self.board.cells[y * self.board.width + x].state == CellState::Hidden {
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

        for (x, y) in self.board.neighbors(x, y) {
            if self.board.cells[y * self.board.width + x].state == CellState::Hidden {
                self.open_cell(x, y);
            }
        }
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        if self.game_over {
            return;
        }
        let cell = &mut self.board.cells[y * self.board.width + x];
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
            self.board.width as f32 * 16.0,
            self.board.height as f32 * 16.0,
        )
    }

    fn opened_cell(count: usize) -> Image<'static> {
        Self::pixelate(Image::new(match count {
            0 => return Minesweeper::empty_cell(),
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

impl Widget for &mut Minesweeper {
    fn ui(self, ui: &mut Ui) -> Response {
        let screen_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_response(screen_rect.size(), egui::Sense::drag());

        let size = Vec2::from(self.size());

        let screen_ratio = screen_rect.aspect_ratio();
        let board_ratio = size.x / size.y;

        let resized = screen_rect.size()
            * vec2(
                (1.0 / (screen_ratio * board_ratio)).min(1.0),
                (screen_ratio * board_ratio).min(1.0),
            );

        let adjusted_rect = Rect::from_min_size(
            screen_rect.min + vec2((screen_rect.size().x - resized.x).max(0.0) / 2.0, 0.0),
            resized,
        );

        let c2s = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, self.size().into()),
            adjusted_rect,
        );

        let mut pressed = None;

        for y in 0..self.board.height {
            for x in 0..self.board.width {
                let rect = Rect::from_min_size(
                    c2s * pos2(x as f32 * 16.0, y as f32 * 16.0),
                    c2s.scale() * vec2(16.0, 16.0),
                );

                let response = ui.interact(rect, response.id.with((x, y)), Sense::click());

                if response.contains_pointer() {
                    if ui.input(|i| i.pointer.primary_down()) {
                        pressed = Some((x, y));
                    }

                    if ui.input(|i| i.pointer.primary_released()) {
                        self.open_cell(x as usize, y as usize);
                    } else if ui.input(|i| i.pointer.secondary_released()) {
                        self.toggle_flag(x as usize, y as usize);
                    }
                }
            }
        }

        for y in 0..self.board.height {
            for x in 0..self.board.width {
                let rect = Rect::from_min_size(
                    c2s * pos2(x as f32 * 16.0, y as f32 * 16.0),
                    c2s.scale() * vec2(16.0, 16.0),
                );

                let cell = &self.board.cells[y * self.board.width + x];

                let image = match (cell.state, cell.kind) {
                    (CellState::Opened, CellKind::Mine) => Minesweeper::opened_mine(),
                    (CellState::Opened, CellKind::Empty) => {
                        Minesweeper::opened_cell(self.board.count_mines(x, y))
                    }

                    (CellState::Flagged, CellKind::Empty) if self.game_over => {
                        Minesweeper::incorrect_flag()
                    }
                    (CellState::Flagged, _) => Minesweeper::flag_cell(),

                    _ if pressed
                        .map(|(px, py)| {
                            (px == x && py == y)
                                || self.board.cells[py * self.board.width + px].state
                                    == CellState::Opened
                                    && x.abs_diff(px) <= 1
                                    && y.abs_diff(py) <= 1
                        })
                        .unwrap_or(false) =>
                    {
                        Minesweeper::empty_cell()
                    }
                    (CellState::Hidden, CellKind::Mine) if self.game_over => {
                        Minesweeper::revealed_mine()
                    }
                    (CellState::Hidden, _) => Minesweeper::hidden_cell(),
                };

                image.paint_at(ui, rect);
            }
        }

        response
    }
}

impl eframe::App for Minesweeper {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}
