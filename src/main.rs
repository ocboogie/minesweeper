use board::Board;
use egui::{Image, ImageSource, TextureOptions};
use minefield::Minefield;

mod board;
mod canvas;
mod minefield;
mod minesweeper;
mod ms_frame;
mod solver;
mod utils;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let game = minesweeper::Minesweeper::new(25, 25, 100);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(game.board.size()),
        ..Default::default()
    };

    eframe::run_native(
        "Minesweeper",
        options,
        Box::new(move |cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(game)
        }),
    )

    // eframe::run_simple_native("Minesweeper", options, move |ctx, _frame| {
    //     egui::CentralPanel::default().show(ctx, move |ui| {
    //         game.ui(ui);
    //     });
    // })
}
