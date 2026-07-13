use std::collections::HashMap;

use eframe::egui::{self, Color32};
use lucide_icons::Icon;
use rand::seq::IndexedRandom;

use crate::lang::*;

struct GameState {
    // maybe should be coupled more
    vars: HashMap<String, f32>,
    var_defs: HashMap<String, usize>, // index to CodeLine
}

struct CodeLine {
    id: usize,
    text: String,
    color: Color32,
    expr: Option<Expr>,
    eval: Option<Eval>,
}

enum CodeAction {
    Insert(usize),
    Remove(usize),
    Focus(usize),
    Run(usize),
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
        first.expr = parse(&first.text).ok();

        let mut app = Self {
            lines: vec![first],
            last_id: 0,
            focus_request: Some(0),
            code_panel_open: true,

            state: GameState { vars: HashMap::new(), var_defs: HashMap::new() },

            pan: egui::vec2(0.0, 0.0),
            zoom: 1.0,
        };

        let eval = app.lines[0]
            .expr
            .as_ref()
            .and_then(|expr| app.eval(expr));
        app.lines[0].eval = eval;
        app
    }
}

impl MyApp {
    fn evalf(&self, expr: &Expr) -> Option<f32> {
        match expr {
            Expr::Bad => None,
            &Expr::Float(f) => Some(f),
            Expr::Name(name) => self.state.vars.get(name).copied(),
            Expr::Call(_) => None,
            Expr::Bin(BinExpr { op, left, right }) => match op {
                BinOp::Add => Some(self.evalf(left)? + self.evalf(right)?),
                BinOp::Sub => Some(self.evalf(left)? - self.evalf(right)?),
                BinOp::Mul => Some(self.evalf(left)? * self.evalf(right)?),
                BinOp::Div => Some(self.evalf(left)? / self.evalf(right)?),
                BinOp::Pow => Some(self.evalf(left)?.powf(self.evalf(right)?)),
                BinOp::Eq => None,
                BinOp::Ne => None,
                BinOp::Lt => None,
                BinOp::Le => None,
                BinOp::Gt => None,
                BinOp::Ge => None,
                BinOp::Arrow => None,
            },
            Expr::Neg(e) => self.evalf(e).map(|f: f32| -f),
            Expr::Factorial(_) => None,
            Expr::Circle(_) => None,
            Expr::Define(_) => None,
            Expr::Assign(_) => None,
        }
    }

    fn eval(&self, expr: &Expr) -> Option<Eval> {
        match expr {
            Expr::Neg(_) | Expr::Float(_) | Expr::Factorial(_) | Expr::Bin(_) | Expr::Call(_) => self.evalf(expr).map(Eval::Float),

            Expr::Bad => None,
            Expr::Name(_) => None,
            Expr::Circle(CircleExpr { x, y, r }) => {
                let x = self.evalf(x)?;
                let y = self.evalf(y)?;
                let r = self.evalf(r)?;
                Some(Eval::Circle(CircleEval { x, y, r }))
            }
            Expr::Define(DefineExpr { name, val }) => {
                let val = self.evalf(val)?;
                Some(Eval::Define(DefineEval { name: name.clone(), val }))
            }
            Expr::Assign(AssignExpr { name, val }) => {
                let val = self.evalf(val)?;
                Some(Eval::Assign(AssignEval { name: name.clone(), val }))
            }
        }
    }

    fn code_eval(&mut self, index: usize) {
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

        let eval = self.eval(expr);
        if let Some(Eval::Define(DefineEval { name, val })) = eval.as_ref() {
            // self.assign(assign.clone());
            if let Some(&def) = self.state.var_defs.get(name) && def != self.lines[index].id {
                // its defined elsewhere
                self.lines[index].eval = None;
            } else {   
                self.state.vars.insert(name.clone(), *val);
                // todo name removing
                self.state.var_defs.insert(name.clone(), self.lines[index].id);
                for i in 0..self.lines.len() {
                    if i == index {
                        continue;
                    }
                    // no overflow bc definition is unique
                    self.code_eval(i);
                }
            }
        }
        self.lines[index].eval = eval;
    }

    fn assign(&mut self, AssignEval { name, val }: AssignEval) {
        // todo not it be lazy
        let Some(&id) = self.state.var_defs.get(&name) else {
            todo!()
        };

        let (index, line) = self.lines.iter_mut().enumerate().filter(|(_, line)| line.id == id).next().unwrap();
        line.text = format!("{name} = {val}");
        line.expr = parse(&line.text).ok();
        self.code_eval(index);
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

                // should this be part of self?
                let mut action = None;
                for i in 0..self.lines.len() {
                    action = action.or(self.show_code_line(i, ui));
                }

                match action {
                    None => {},
                    Some(CodeAction::Insert(index)) => self.insert(index),
                    Some(CodeAction::Remove(index)) => {
                        let line = self.lines.remove(index);

                        // todo add proper remove procedure
                        let mut rem_name = None;
                        for (name, &id) in &self.state.var_defs {
                            if line.id == id {
                                rem_name = Some(name.clone());
                            }
                        }
                        if let Some(name) = rem_name {
                            self.state.var_defs.remove(&name);
                        }

                        self.focus_request = Some(self.lines[index - 1].id);
                    },
                    Some(CodeAction::Focus(index)) => self.focus_request = Some(self.lines[index].id),
                    Some(CodeAction::Run(index)) => {
                        let Some(Eval::Assign(assign)) = &self.lines[index].eval else {
                            panic!("this shouldnt be possible");
                        };
                        self.assign(assign.clone());
                    }
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

impl MyApp {
    fn show_code_line(&mut self, index: usize, ui: &mut egui::Ui) -> Option<CodeAction> {
        let line = &mut self.lines[index];
        let was_empty = line.text.is_empty();
        enum Response {
            Egui(egui::Response),
            Run,
            ParseEval,
        }
        let response = ui
            .push_id(line.id, |ui| {
                let response = ui.horizontal(|ui| {
                    match line.eval {
                        Some(Eval::Circle(_)) => {
                            ui.color_edit_button_srgba(&mut line.color);
                        },
                        Some(Eval::Assign(_)) => {
                            if ui.button("->").clicked() {
                                return Response::Run;
                            }
                        },
                        _ => {},
                    };
                    let response = ui.text_edit_singleline(&mut line.text);
                    Response::Egui(response)
                }).inner;
                let response = match response {
                    Response::Egui(response) => response,
                    Response::Run => return Response::Run,
                    Response::ParseEval => return Response::ParseEval,
                };
                if response.changed() {
                    return Response::ParseEval;
                }

                if let Some(Eval::Define(DefineEval { name, val })) = &line.eval {
                    let mut val = *val;
                    if ui.add(egui::Slider::new(&mut val, -10.0..=10.0)).changed() {
                        line.text = format!("{name} = {val}");
                        return Response::ParseEval;
                    }
                }

                if let Some(expr) = &line.expr {
                    ui.label(format!("{:?}", expr));
                }        
                if let Some(eval) = &line.eval {
                    if ui.button(format!("{:?}", eval)).clicked() {
                        return Response::ParseEval;
                    }
                }
                Response::Egui(response)
            })
            .inner;
        let response = match response {
            Response::Egui(response) => response,
            Response::Run => return Some(CodeAction::Run(index)),
            Response::ParseEval => {
                line.expr = parse(&line.text).ok();
                self.code_eval(index);
                return None;
            },
        };
        
        if let Some(focus_request) = self.focus_request && line.id == focus_request {
            response.request_focus();
            self.focus_request = None;
        }
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            return Some(CodeAction::Insert(index));
        }
        if index > 0 && response.has_focus() && was_empty && ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
            return Some(CodeAction::Remove(index));
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