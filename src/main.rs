mod board;
mod canvas;
mod minefield;
mod minesweeper;
mod ms_button;
mod ms_frame;
mod ms_modal;
mod solver;
mod utils;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    use minesweeper::Minesweeper;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // let game = minesweeper::Minesweeper::new(10, 10, 16);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Minesweeper",
        options,
        Box::new(move |cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(Minesweeper::new(&cc.egui_ctx))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);

                    Box::new(Minesweeper::new(&cc.egui_ctx))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
