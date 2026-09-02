use std::collections::HashMap;

use eframe::egui::{self, Color32};
use lucide_icons::Icon;
use rand::seq::IndexedRandom;
use serde_json::json;

use crate::{eval::*, parse::*};

struct CodeLine {
    id: usize,
    text: String,
    color: Color32,
    expr: Option<Expr>,
    func: Option<FunctionEval>,
    eval: Option<Eval>,
}

enum CodeAction {
    Insert(usize),
    Remove(usize),
    Focus(usize),
    ParseEval(usize),
    Run(usize),
}

struct MyApp {
    lines: Vec<CodeLine>,
    last_id: usize,
    focus_request: Option<usize>,
    code_panel_open: bool,

    // todo add proper gamestate
    vars: HashMap<String, usize>,

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
            func: None,
            eval: None,
        };
        first.expr = parse(&first.text).ok();

        let mut app = Self {
            lines: vec![first],
            last_id: 0,
            focus_request: Some(0),
            code_panel_open: true,

            vars: HashMap::new(),

            pan: egui::vec2(0.0, 0.0),
            zoom: 1.0,
        };

        let eval = app.lines[0]
            .expr
            .as_ref()
            .and_then(|expr| eval(expr, &app).ok());
        app.lines[0].eval = eval;
        app
    }
}

impl Args for MyApp {
    fn get(&self, name: &String) -> Option<Eval> {
        let &line_id = self.vars.get(name)?;
        let def_eval = self.lines.iter().filter(|line| line.id == line_id).next().and_then(|line| line.eval.clone())?;
        let Eval::Define(DefineEval { val, .. }) = def_eval else { unreachable!() };
        Some(*val)
    }
}

impl MyApp {
    fn code_parse(&mut self, index: usize) {
        self.lines[index].expr = parse(&self.lines[index].text).ok();
    }

    fn code_eval(&mut self, index: usize) {
        let Some(expr) = self.lines[index].expr.as_ref() else {
            self.lines[index].func = None;
            self.lines[index].eval = None;
            return;
        };

        let Ok(func_eval) = eval(expr, &()) else {
            self.lines[index].func = None;
            self.lines[index].eval = None;
            return;
        };

        if let Eval::Function(func) = func_eval {
            self.lines[index].func = Some(func);
            let eval = eval(&self.lines[index].func.as_ref().unwrap().inner, self);
            self.lines[index].eval = eval.ok();
        } else {
            self.lines[index].eval = Some(func_eval);
        };

        if let Some(Eval::Define(DefineEval { name, .. })) = self.lines[index].eval.as_ref() {
            // self.assign(assign.clone());
            if let Some(&line_id) = self.vars.get(name) && line_id != self.lines[index].id {
                // its defined elsewhere
                self.lines[index].eval = None;
                println!("alr defined elsewhere");
            } else {
                self.vars.insert(name.clone(), self.lines[index].id);
                let reval: Vec<usize> = self.lines.iter().enumerate()
                    .filter(|(i, line)| {
                        *i != index &&
                        if let Some(FunctionEval { inner: _, params }) = &line.func {
                            params.contains(name)
                        } else { false }
                    }).map(|(index, _)| index).collect();
                for index in reval {
                    dbg!(index);
                    self.code_eval(index);
                }
            }
        }
    }

    fn assign(&mut self, AssignEval { name, val }: AssignEval) {
        let index = if let Some(&line_id) = self.vars.get(&name) {
            self.lines.iter().enumerate().filter(|(_, line)| line.id == line_id).next().unwrap().0
        } else {
            let index = self.lines.len();
            self.insert(index - 1);
            index
        };
        self.lines[index].text = format!("{name} = {val:?}");
        self.code_parse(index);
        self.code_eval(index);
    }

    fn insert(&mut self, after_index: usize) {
        self.last_id += 1;
        self.lines.insert(
            after_index + 1,
            CodeLine {
                id: self.last_id,
                text: String::new(),
                color: rand_color(),
                expr: None,
                func: None,
                eval: None,
            },
        );
        self.focus_request = Some(self.last_id);
    }

    fn remove(&mut self, index: usize) {
        let line = self.lines.remove(index);
        
        // todo myb make more robust
        if let Some(Eval::Define(DefineEval { name, .. })) = line.eval {
            self.vars.remove(&name);
        }
        
        self.focus_request = Some(self.lines[index - 1].id);
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let heading_res = egui::Panel::top("header").show(ui, |ui| {
            ui.heading("Engine");
        });

        let mut code_panel_open = self.code_panel_open;
        if code_panel_open {
            egui::Panel::left("code_edit").show(ui, |ui| {
                egui::Sides::new().show(ui, |ui| {
                    if ui.button(String::from(char::from(Icon::Share))).clicked() {
                        println!("{}", self.to_json());
                    }
                }, |ui| {
                    if ui.button(String::from(char::from(Icon::PanelLeftClose))).clicked() {
                        code_panel_open = false;
                    }
                });

                // should this be part of self?
                let mut action = None;
                for i in 0..self.lines.len() {
                    action = action.or(self.show_code_line(i, ui));
                }

                match action {
                    None => {},
                    Some(CodeAction::Insert(index)) => self.insert(index),
                    Some(CodeAction::Remove(index)) => self.remove(index),
                    Some(CodeAction::Focus(index)) => self.focus_request = Some(self.lines[index].id),
                    Some(CodeAction::ParseEval(index)) => {
                        self.code_parse(index);
                        self.code_eval(index);
                    },
                    Some(CodeAction::Run(index)) => {
                        let Some(Eval::Assign(assign)) = &self.lines[index].eval else {
                            panic!("this shouldnt be possible");
                        };
                        self.assign(assign.clone());
                    }
                }
            });
            self.code_panel_open = code_panel_open;
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

impl MyApp {
    fn show_code_line(&mut self, index: usize, ui: &mut egui::Ui) -> Option<CodeAction> {
        let line = &mut self.lines[index];
        enum Response {
            Egui(egui::Response),
            CodeAction(CodeAction),
        }
        let response = ui
            .push_id(line.id, |ui| {
                let was_empty = line.text.is_empty();
                let response = ui.horizontal(|ui| {
                    match line.eval {
                        Some(Eval::Circle(_)) => {
                            ui.color_edit_button_srgba(&mut line.color);
                        },
                        Some(Eval::Assign(_)) => {
                            if ui.button(String::from(char::from(Icon::Play))).clicked() {
                                return Response::CodeAction(CodeAction::Run(index));
                            }
                        },
                        _ => {},
                    };
                    let response = ui.text_edit_singleline(&mut line.text);
                    Response::Egui(response)
                }).inner;
                let response = match response {
                    Response::Egui(response) => response,
                    _ => return response,
                };
                if response.changed() {
                    if index > 0 && response.has_focus() && was_empty && ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
                        println!("remove");
                        return Response::CodeAction(CodeAction::Remove(index));
                    }
                    return Response::CodeAction(CodeAction::ParseEval(index));
                }

                if let Some(Eval::Define(DefineEval { name, val })) = &line.eval && let Eval::Float(val) = **val {
                    let mut val = val;
                    if ui.add(egui::Slider::new(&mut val, -10.0..=10.0)).changed() {
                        line.text = format!("{name} = {val}");
                        return Response::CodeAction(CodeAction::ParseEval(index));
                    }
                }

                if let Some(expr) = &line.expr {
                    ui.label(format!("{:?}", expr));
                }
                if let Some(func) = &line.func {
                    ui.label(format!("{:?}", func));
                }
                if let Some(eval) = &line.eval {
                    if ui.button(format!("{:?}", eval)).clicked() {
                        return Response::CodeAction(CodeAction::ParseEval(index));
                    }
                }
                Response::Egui(response)
            })
            .inner;
        let response = match response {
            Response::Egui(response) => response,
            Response::CodeAction(action) => return Some(action),
        };
        
        if let Some(focus_request) = self.focus_request && line.id == focus_request {
            response.request_focus();
            self.focus_request = None;
        }
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            return Some(CodeAction::Insert(index));
        }
        if response.has_focus() && index > 0 && ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            return Some(CodeAction::Focus(index - 1));
        }
        if response.has_focus() && index < self.lines.len() - 1 && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            return Some(CodeAction::Focus(index + 1));
        }

        return None;
    }
}

impl MyApp {
    fn to_json(&self) -> String {
        let lines: Vec<serde_json::Value> = self.lines.iter().map(|line| {
            json!({
                "type": "expression",
                "id": line.id,
                "latex": line.text,
                "color": line.color.to_hex(),
            })
        }).collect();

        let state = json!({
            "graph": {
                "viewport": {
                    "xmin": -10,
                    "xmax": 10,
                    "ymin": 0,
                    "ymax": 0,
                },
            },
            "expressions": {
                "list": lines,
            },
        });
        
        state.to_string()
    }
}

fn rand_color() -> Color32 {
    *[Color32::RED, Color32::ORANGE, Color32::YELLOW, Color32::GREEN, Color32::CYAN, Color32::BLUE, Color32::PURPLE].choose(&mut rand::rng()).unwrap()
}

// gemini
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