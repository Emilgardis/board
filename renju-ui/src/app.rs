

use egui::{Button, TextEdit, Widget};
use poll_promise::Promise;


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
    #[serde(skip)]
    just_clicked: bool,
}

impl Default for RenjuApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            board: UIBoard::new(),
            picker_promise: None,
            just_clicked: false,
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
    #[tracing::instrument(skip(self, ctx, frame), fields(move_list = ?self.board.graph().move_list()))]
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            label: _,
            value: _,
            board,
            picker_promise,
            just_clicked,
        } = self;

        if *just_clicked {
            *just_clicked = false;

            tracing::info!(moves = ?board.moves(), variants = ?board.variants());
        }
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
                ui.menu_button("Transform", |ui| {
                    if ui.button("rotate").clicked() {
                        board
                            .transform_mut()
                            .rotate(renju::board::Rotation::Deg90);
                    }
                    if ui.button("Mirror -").clicked() {
                        board.transform_mut().mirror = renju::board::Mirror::Horizontal;
                    }
                    if ui.button("Mirror |").clicked() {
                        board.transform_mut().mirror = renju::board::Mirror::Vertical;
                    }
                    if ui.button("Mirror None").clicked() {
                        board.transform_mut().mirror = renju::board::Mirror::None;
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
                                let curr_move = board.graph().current_move();
                                renju::file_reader::read_bytes(
                                    bytes.as_slice(),
                                    Some(&renju::file_reader::FileType::Lib),
                                    board.graph_mut(),
                                )
                                .unwrap();

                                board.change_current_move(&curr_move);
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
                let moves = board
                    .graph()
                    .move_list()
                    .iter()
                    .filter_map(|i| board.graph().get_move(*i))
                    .collect::<Vec<_>>();
                let moves_i = board.graph().move_list().iter().collect::<Vec<_>>();
                ui.text_edit_multiline(&mut format!("Moves: {moves:?}"));
                ui.text_edit_multiline(&mut format!("Moves I: {moves_i:?}"));
                ui.text_edit_multiline(&mut format!("Transform: {:?}", board.transform()));

                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    let current = board.current_move_mut();
                    let multi = current.multiline_comment.as_mut();
                    ui.label("Comment:");
                    match multi {
                        Some(multi) if !multi.is_empty() => {
                            ui.text_edit_multiline(multi);
                        }
                        _ => {
                            let mut multi = String::new();
                            TextEdit::multiline(&mut multi).hint_text("...").ui(ui);
                            current.set_multiline_comment(multi);
                        }
                    }

                    let one = current.oneline_comment.as_mut();
                    ui.label("Comment:");
                    match one {
                        Some(one) if !one.is_empty() => {
                            ui.text_edit_multiline(one);
                        }
                        _ => {
                            let mut one = String::new();
                            TextEdit::singleline(&mut one).hint_text("...").ui(ui);
                            current.set_oneline_comment(one);
                        }
                    }

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

        egui::CentralPanel::default().show(ctx, |ui| board.ui(ui, just_clicked));

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
