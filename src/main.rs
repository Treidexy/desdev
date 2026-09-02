#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::{debug, error, info, warn};

mod app;
mod parse;
mod eval;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();
    debug!("hi");
    error!("hola");
    warn!("bonjour");
    info!(":)");
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "desdev",
        options,
        Box::new(app::creator),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        use eframe::wasm_bindgen::JsCast;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas: web_sys::HtmlCanvasElement = document.get_element_by_id("the_canvas_id").unwrap().dyn_into().unwrap();
        let start_result = eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(app::creator))
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_thing") {
            match start_result {
                Ok(()) => {
                    loading_text.remove();
                }
                Err(err) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {err:?}");
                }
            }
        }
    });
}