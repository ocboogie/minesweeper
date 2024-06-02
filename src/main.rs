use egui::{Image, ImageSource, TextureOptions};

mod board;
mod board_ui;
mod canvas;
mod minesweeper;
mod ms_frame;

pub fn load_image(src: ImageSource) -> Image<'_> {
    Image::new(src)
        .fit_to_original_size(1.0)
        .texture_options(TextureOptions::NEAREST)
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let game = minesweeper::Minesweeper::new(16, 16, 40);

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
