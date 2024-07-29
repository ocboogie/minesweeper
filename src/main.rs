mod board;
mod canvas;
mod generating;
mod minefield;
mod minesweeper;
mod ms_button;
mod ms_frame;
mod ms_modal;
mod rref;
mod solver;
mod utils;

use minesweeper::Minesweeper;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    use minefield::Minefield;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // let game = minesweeper::Minesweeper::new(10, 10, 16);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    let minefield = Minefield::parse(
        r#"m.1m.1......
22112F22m..m
F100223m....
11001F3.m...
001134F42...
124F4FF3F2..
1FFFF43223m.
12333F101F..
1100111022.m
F21000001F3.
3F100011213F
F310002F202m
F200002F201."#,
    );
    dbg!(&minefield);
    eframe::run_native(
        "Minesweeper",
        options,
        Box::new(move |cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(Minesweeper::new_beginner(&cc.egui_ctx))
            // Box::new(Minesweeper::start_from_minefield(&cc.egui_ctx, minefield))
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
