mod board;
mod canvas;
mod minefield;
mod minesweeper;
mod ms_button;
mod ms_frame;
mod ms_modal;
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
        r#"m.1m.1......mm.m........m.....
22112F..m..m...m.....m........
F10022.m...........m.m..m.....
11001F..m....m.m...m....mm....
001134m......m......m....mm...
124F4FF3F2...m.mmmm........m..
1FFFF32212m....m.m....m..m..m.
1233321012......mm.m......m...
00001F101F..mmm.m.....m.......
1100111022.m..................
F21000001F3..m.mm.............
3F100011213m..m.............m.
F310002F202m........mmm.m....m
F322224F323...............m..m
12FF3FF3.mF.mmm.m.......m..m.."#,
    );
    dbg!(&minefield);
    eframe::run_native(
        "Minesweeper",
        options,
        Box::new(move |cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(Minesweeper::start_from_minefield(&cc.egui_ctx, minefield))
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
