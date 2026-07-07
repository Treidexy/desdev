use std::collections::HashMap;

use eframe::egui::{self, Color32};
use lucide_icons::Icon;
use rand::seq::IndexedRandom;

use crate::lang::*;

struct GameState {
    vars: HashMap<String, f32>,
}

struct CodeLine {
    id: usize,
    text: String,
    color: Color32,
    expr: Option<Expr>,
    eval: Option<Eval>,
}

struct MyApp {
    lines: Vec<CodeLine>,
    last_id: usize,
    focus_request: Option<usize>,
    code_panel_open: bool,

    state: GameState,

    pan: egui::Vec2,
    zoom: f32,
}

pub fn creator(cc: &eframe::CreationContext) -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
    setup_fonts(&cc.egui_ctx);
    Ok(Box::<MyApp>::default())
}

impl Default for MyApp {
    fn default() -> Self {
        let mut first = CodeLine {
            id: 0,
            text: "circle(0, 0, 25)".to_owned(),
            color: rand_color(),
            expr: None,
            eval: None,
        };
        first.parse();
        // first.eval = eval(first.expr.as_ref().unwrap());

        Self {
            lines: vec![first],
            last_id: 0,
            focus_request: Some(0),
            code_panel_open: true,

            state: GameState { vars: HashMap::new() },

            pan: egui::vec2(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl CodeLine {
    fn parse(&mut self) {
        self.expr = parse(&self.text).ok();
    }   
}

impl MyApp {
    fn eval(&mut self, index: usize) {
        let Some(expr) = self.lines[index].expr.as_ref() else {
            self.lines[index].eval = None;
            return;
        };

        match expr {
            Expr::Name(name) => {
                let eval = self.state.vars.get(name).map(|&v| Eval::Float(v));
                self.lines[index].eval = eval;
                return;
            },
            _ => {},
        }

        let eval = eval(expr);
        if let Some(Eval::Assign(AssignEval { name, val })) = eval.as_ref() {
            self.state.vars.insert(name.clone(), *val);
        }
        self.lines[index].eval = eval;
    }

    fn insert(&mut self, index: usize) {
        self.last_id += 1;
        self.lines.insert(
            index + 1,
            CodeLine {
                id: self.last_id,
                text: String::new(),
                color: rand_color(),
                expr: None,
                eval: None,
            },
        );
        self.focus_request = Some(self.last_id);
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let heading_res = egui::Panel::top("header").show(ui, |ui| {
            ui.heading("Engine");
        });

        if self.code_panel_open {
            egui::Panel::left("code_edit").show(ui, |ui| {
                egui::Sides::new().show(ui, |_ui| {}, |ui| {
                    if ui.button(String::from(char::from(Icon::PanelLeftClose))).clicked() {
                        self.code_panel_open = false;
                    }
                });

                enum Action {
                    None,
                    Insert(usize),
                    Remove(usize),
                    Focus(usize),
                    Eval(usize),
                }
                let mut action = Action::None;

                let lines_len = self.lines.len();
                for (i, line) in self.lines.iter_mut().enumerate() {
                    let was_empty = line.text.is_empty();
                    let response = ui
                        .push_id(line.id, |ui| ui.add(CodeLineWidget(line)))
                        .inner;
                    if let Some(focus_request) = self.focus_request && line.id == focus_request {
                        response.request_focus();
                        self.focus_request = None;
                    }
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        action = Action::Insert(i);
                    }
                    if i > 0 && response.has_focus() && was_empty && ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
                        action = Action::Remove(i);
                    }
                    if response.has_focus() && i > 0 && ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        action = Action::Focus(i - 1);
                    }
                    if response.has_focus() && i < lines_len - 1 && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        action = Action::Focus(i + 1);
                    }
                    if response.changed() {
                        action = Action::Eval(i);
                    }
                }

                match action {
                    Action::None => {},
                    Action::Insert(index) => self.insert(index),
                    Action::Remove(index) => {
                        self.lines.remove(index);
                        self.focus_request = Some(self.lines[index - 1].id);
                    },
                    Action::Focus(index) => self.focus_request = Some(self.lines[index].id),
                    Action::Eval(index) => self.eval(index),
                }
            });
        } else {
            egui::Area::new(egui::Id::new("top_left_area"))
                .fixed_pos(heading_res.response.rect.left_bottom() + egui::vec2(10.0, 10.0))
                // .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
                .show(ui, |ui| {
                    if ui.button(String::from(char::from(Icon::PanelLeftOpen))).clicked() {
                        self.code_panel_open = true;
                    }
                });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::click_and_drag()
            );

            // painter.rect_filled(response.rect, 0.0, egui::Color32::from_rgb(64, 64, 64));
            // painter.circle_filled(response.rect.center(), 67.0, egui::Color32::RED);

            if response.dragged() {
                self.pan += response.drag_delta();
            }

            if response.hovered() {
                // 1. Get native zoom gesture (works for touchscreens / macOS trackpads)
                let mut zoom_factor = ui.input(|i| i.zoom_delta());

                // 2. Fallback for Windows/Linux trackpads (Ctrl + Scroll)
                let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
                
                if scroll_y != 0.0 {
                    // Translate the emulated scroll into a zoom factor
                    zoom_factor = (scroll_y * 0.005).exp();
                }

                // 3. Apply the zoom if either trigger happened
                if zoom_factor != 1.0 {
                    let old_zoom = self.zoom;
                    self.zoom *= zoom_factor;
                    // self.zoom = self.zoom.clamp(0.1, 10.0);
                    
                    let actual_zoom_factor = self.zoom / old_zoom;

                    if let Some(pointer_pos) = response.hover_pos() {
                        let pointer_offset = pointer_pos - response.rect.center();
                        self.pan = pointer_offset - (pointer_offset - self.pan) * actual_zoom_factor;
                    }
                }
            }

            let transform_to_screen = |world_pos: egui::Pos2| -> egui::Pos2 {
                let scaled = world_pos.to_vec2() * self.zoom;
                response.rect.center() + scaled + self.pan
            };

            for line in &self.lines {
                if let Some(Eval::Circle(circle)) = &line.eval {
                    painter.circle_filled(
                        transform_to_screen(egui::pos2(circle.x, circle.y)),
                        circle.r * self.zoom,
                        line.color);
                }
            }

            if let Some(pointer_pos) = response.hover_pos() {
                painter.circle_stroke(
                    pointer_pos,
                    5.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                );
            }

            if response.clicked() {
                println!("Canvas clicked at: {:?}", response.interact_pointer_pos());
            }
        });

        egui::Area::new(egui::Id::new("top_right_area"))
            .fixed_pos(heading_res.response.rect.right_bottom() + egui::vec2(-30.0, 10.0))
            // .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
            .layout(egui::Layout::top_down(egui::Align::Max))
            .show(ui, |ui| {
                if ui.button(String::from(char::from(Icon::Home))).clicked() {
                    self.pan = egui::vec2(0.0, 0.0);
                    self.zoom = 1.0;
                }

                if ui.button(String::from(char::from(Icon::Plus))).clicked() {
                    self.zoom *= 2.0;
                    self.pan *= 2.0;
                }
                
                if ui.button(String::from(char::from(Icon::Minus))).clicked() {
                    self.zoom /= 2.0;
                    self.pan /= 2.0;
                }
            });
    }
}

struct CodeLineWidget<'a>(&'a mut CodeLine);

impl<'a> egui::Widget for CodeLineWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut self.0.color);
            let response = ui.text_edit_singleline(&mut self.0.text);

            response
        }).inner;
        
        if response.changed() {
            self.0.parse();
        }

        if let Some(expr) = &self.0.expr {
            ui.label(format!("{:?}", expr));
        }        
        if let Some(eval) = &self.0.eval {
            let _ = ui.button(format!("{:?}", eval));
        }
        
        response
    }
}


fn rand_color() -> Color32 {
    *[Color32::RED, Color32::ORANGE, Color32::YELLOW, Color32::GREEN, Color32::CYAN, Color32::BLUE, Color32::PURPLE].choose(&mut rand::rng()).unwrap()
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 1. Add the Lucide font data
    fonts.font_data.insert(
        "lucide".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(lucide_icons::LUCIDE_FONT_BYTES)),
    );
    
    // 2. Tell egui to use it as a fallback for standard text
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("lucide".to_owned());
        

    ctx.set_fonts(fonts);
}