use log::{debug, error, info, warn};

mod app;
mod lang;

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