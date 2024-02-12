#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(renju_ui::RenjuApp::new(cc))),
    );
    Ok(())
}
