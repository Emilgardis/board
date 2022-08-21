use egui::{Button, RawInput};
use poll_promise::Promise;
use renju::board_logic::BoardArr;

use crate::board::UIBoard;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RenjuApp {
    // Example stuff:
    label: String,
    #[serde(skip)]
    board: super::board::UIBoard,
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,
    #[serde(skip)]
    picker_promise: Option<Promise<Option<Vec<u8>>>>,
}

impl Default for RenjuApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            board: UIBoard::new(),
            picker_promise: None,
        }
    }
}

impl RenjuApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for RenjuApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            label,
            value,
            board,
            picker_promise,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Side Panel");

                ui.horizontal(|ui| {
                    ui.label("Write something: ");
                    let prev = board.graph().prev_move();

                    if let Some(promise) = picker_promise.take() {
                        match promise.poll() {
                            std::task::Poll::Ready(Some(bytes)) => {
                                // lol
                                renju::file_reader::read_bytes(
                                    bytes.as_slice(),
                                    Some(&renju::file_reader::FileType::Lib),
                                    board.graph_mut(),
                                )
                                .unwrap();
                            }
                            std::task::Poll::Ready(None) => (),
                            _ => {
                                picker_promise.replace(promise);
                            }
                        }
                    }
                    if ui
                        .add_enabled(picker_promise.is_none(), Button::new("load file"))
                        .clicked()
                    {
                        *picker_promise = Some(poll_promise::Promise::spawn_async(async move {
                            match rfd::AsyncFileDialog::new().pick_file().await {
                                Some(fh) => Some(fh.read().await),
                                None => None,
                            }
                        }));
                    }
                    if ui
                        .add_enabled(prev.is_some(), Button::new("back"))
                        .clicked()
                    {
                        board.change_current_move(&prev.unwrap())
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        if cfg!(debug_assertions) {
                            ui.label(
                                egui::RichText::new("‼ Debug build ‼")
                                    .small()
                                    .color(egui::Color32::RED),
                            )
                            .on_hover_text("renju_ui was compiled with debug assertions enabled.");
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| board.ui(ui));

        if true {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
