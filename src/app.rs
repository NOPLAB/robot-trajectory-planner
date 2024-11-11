use std::vec;

use egui::{emath, Pos2, Rect, Sense, Shape, Stroke, Vec2};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    label: String,

    #[serde(skip)]
    lines: Vec<Pos2>,

    #[serde(skip)]
    stroke: Stroke,

    #[serde(skip)]
    t: f32,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            label: "Robot Trajectory Planner".to_owned(),
            lines: vec![],
            stroke: Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 0)),
            t: 0.0,
            value: 2.7,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (mut response, painter) =
                    ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

                let to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
                    response.rect,
                );
                let from_screen = to_screen.inverse();

                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let canvas_pos = from_screen * pointer_pos;
                    self.lines.push(canvas_pos);
                    response.mark_changed();
                } else {
                    response.mark_changed();
                }

                let mut shapes = if self.lines.len() >= 2 {
                    let lines = self
                        .lines
                        .iter()
                        .map(|p| to_screen * *p)
                        .collect::<Vec<Pos2>>();
                    vec![Shape::line(lines, self.stroke)]
                } else {
                    vec![]
                };

                let time = ui.input(|i| i.time) * 0.1;
                if let Some(result) = line_pos(self.lines.clone(), time as f32) {
                    shapes.push(Shape::circle_filled(
                        to_screen * result,
                        10.0,
                        egui::Color32::from_rgb(255, 255, 255),
                    ));

                    if let Some(p) = line_pos(self.lines.clone(), time as f32 + 0.02) {
                        let delta = Pos2::new(p.x - result.x, p.y - result.y);

                        let theta = delta.y.atan2(delta.x);

                        println!("theta: {}", theta);

                        shapes.push(Shape::line(
                            vec![
                                to_screen * result,
                                to_screen
                                    * Pos2::new(
                                        result.x + 2.0 * theta.cos(),
                                        result.y + 2.0 * theta.sin(),
                                    ),
                            ],
                            Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 255)),
                        ));
                    }
                }

                painter.extend(shapes);

                response
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

fn line_pos(lines: Vec<Pos2>, time: f32) -> Option<Pos2> {
    if lines.len() < 2 {
        return None;
    }

    let mut i = 1;
    let mut prev_total_len = 0.0;
    let mut total_len = 0.0;
    let mut prev_pos = lines.get(0).unwrap();
    let mut result = Pos2::new(0.0, 0.0);
    loop {
        if let Some(new_pos) = lines.get(i) {
            let delta = Pos2::new(new_pos.x - prev_pos.x, new_pos.y - prev_pos.y);
            total_len += (delta.x.powi(2) + delta.y.powi(2)).sqrt();

            if time >= prev_total_len && total_len >= time {
                // new_posとprev_posの間をtだけ進んだ位置を求める
                let t = (time - prev_total_len) / (total_len - prev_total_len);
                let x = prev_pos.x + (new_pos.x - prev_pos.x) * t;
                let y = prev_pos.y + (new_pos.y - prev_pos.y) * t;

                return Some(Pos2::new(x, y));
            }

            prev_pos = new_pos;
            prev_total_len = total_len;
            i += 1;
        } else {
            return None;
        }
    }
}
