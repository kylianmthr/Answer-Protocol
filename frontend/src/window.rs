use std::sync::Arc;

use crate::auth::auth;
use crate::parser::ServerMessage;
use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily, Ui};
use egui_notify::Toasts;

// init screen egui(GUI)
//app::MyTape
pub struct MyTap {
    screen: Screen,
}

// default start program into login page
// modify into update fn with self.screen to change state
//e.g self.screen = Screen::GameView{...}
impl MyTap {
    pub fn new(
        rx_incoming: std::sync::mpsc::Receiver<ServerMessage>,
        tx_outgoing: std::sync::mpsc::Sender<String>,
    ) -> Self {
        Self {
            screen: Screen::LoginView(LoginPage::new(rx_incoming, tx_outgoing)),
        }
    }
}

// mult screen manager
enum Screen {
    LoginView(LoginPage),
    GameView(GamePage),
}

// macro to define default field with given type username == ""
struct LoginPage {
    username: String,
    rx_incoming: std::sync::mpsc::Receiver<ServerMessage>,
    tx_outgoing: std::sync::mpsc::Sender<String>,
    serveur_adrr: String, // sock into str -> .parse() convert to u16
    toasts: Toasts,
    waiting_res: bool,
}

impl LoginPage {
    pub fn new(
        rx_incoming: std::sync::mpsc::Receiver<ServerMessage>,
        tx_outgoing: std::sync::mpsc::Sender<String>,
    ) -> Self {
        Self {
            username: String::new(),
            rx_incoming,
            tx_outgoing,
            serveur_adrr: String::new(),
            toasts: Toasts::default(),
            waiting_res: false,
        }
    }
}

struct GamePage {
    username: String,
}

// cas ou tu veux ajouter des font cas specifique
// pub struct Font {
// 	undertale_font: String,
// }

pub fn font_style(egui_ctx: &egui::Context) {
    let mut undertale_font = FontDefinitions::default();

    undertale_font.font_data.insert(
        "undertale_font".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../font/undertale_font.ttf"
        ))),
    );

    undertale_font.families.insert(
        FontFamily::Name("undertale_font".into()),
        vec!["undertale_font".to_owned()],
    );

    //font priority projet
    undertale_font
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "undertale_font".to_owned());

    // security_font
    undertale_font
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push(("undertale_font").to_owned());

    egui_ctx.set_fonts(undertale_font);
}

impl MyTap {
    fn draw_field_log(ui: &mut egui::Ui, login_page: &mut LoginPage) {
        ui.vertical_centered(|ui| {
            ui.add_space(250.0);

            ui.scope(|ui| {
                let style_field = ui.style_mut();
                let rounding_field = egui::CornerRadius::same(10_u8);

                style_field.visuals.extreme_bg_color = egui::Color32::WHITE;
                style_field.visuals.override_text_color = Some(egui::Color32::BLACK);

                style_field.visuals.widgets.active.corner_radius = rounding_field;
                style_field.visuals.widgets.hovered.corner_radius = rounding_field;
                style_field.visuals.widgets.inactive.corner_radius = rounding_field;
                style_field.override_font_id = Some(egui::FontId::proportional(24.0_f32));
                style_field.visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;

                ui.add(
                    egui::TextEdit::singleline(&mut login_page.username)
                        .hint_text("Username:")
                        .font(egui::FontId::new(
                            20.0_f32,
                            egui::FontFamily::Name("undertale_font".into()),
                        )),
                );
            });

            ui.add_space(42.0);
            ui.scope(|ui| {
                if ui.button("Login").clicked() {
                    login_page
                        .tx_outgoing
                        .send(format!("CONNECT {}", login_page.username))
                        .unwrap();
                    login_page.waiting_res = true;
                    //match auth(
                    //    &login_page.rx_incoming,
                    //    &login_page.tx_outgoing,
                    //    login_page.username.clone(),
                    //) {
                    //    Ok(_) => {
                    //        login_page.toasts.success("Login successful".to_string());
                    //        println!("Login successful");
                    //    }
                    //    Err(e) => {
                    //        println!("Login failed: {}", e);
                    //        login_page.toasts.error(format!("Login failed: {}", e));
                    //    }
                    //}
                }
            });
        });
    }

    fn draw_test(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(250.0);
            ui.scope(|ui| {
                let style_field = ui.style_mut();
                let rounding_field = egui::CornerRadius::same(10_u8);

                style_field.visuals.extreme_bg_color = egui::Color32::WHITE;
                style_field.visuals.override_text_color = Some(egui::Color32::BLACK);

                style_field.visuals.widgets.active.corner_radius = rounding_field;
                style_field.visuals.widgets.hovered.corner_radius = rounding_field;
                style_field.visuals.widgets.inactive.corner_radius = rounding_field;
                style_field.override_font_id = Some(egui::FontId::proportional(24.0_f32));
                style_field.visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;

                ui.add(
                    egui::TextEdit::singleline(&mut String::new())
                        .hint_text("Username:")
                        .font(egui::FontId::new(
                            20.0_f32,
                            egui::FontFamily::Name("undertale_font".into()),
                        )),
                );
            });
        });
    }
}

// apply contrat (App) on MyTap
impl eframe::App for MyTap {
    // modify (mut) once per frame
    fn ui(&mut self, ctx: &mut Ui, _frame: &mut eframe::Frame) {
        let remove_border_bg =
            egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(0));
        if matches!(&self.screen, Screen::GameView(_)) {
            egui::SidePanel::right("chat_panel")
                .min_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Chat");
                    // todo!() // chat panel
                });
        }
        egui::CentralPanel::default()
            .frame(remove_border_bg)
            .show(ctx, |ui| {
                let image_log_bg = egui::include_image!("../asset_manager/asset_log.jpeg");
                let get_rect_screen = ui.max_rect(); // window_size

                egui::Image::new(image_log_bg).paint_at(ui, get_rect_screen);
                match &mut self.screen {
                    Screen::LoginView(login_page) => {
                        Self::draw_field_log(ui, login_page);
                    }
                    Screen::GameView(game_page) => {
                        Self::draw_test(ui);
                    }
                };
            });

        let mut transition: Option<Screen> = None;

        if let Screen::LoginView(login_page) = &mut self.screen {
            login_page.toasts.show(ctx);

            if login_page.waiting_res {
                match login_page.rx_incoming.try_recv() {
                    Ok(ServerMessage::Ok(_)) => {
                        login_page.waiting_res = false;
                        login_page.toasts.success("Login successful".to_string());
                        transition = Some(Screen::GameView(GamePage {
                            username: login_page.username.clone(),
                        }));
                    }
                    Ok(ServerMessage::Err { code: 500, message }) => {
                        login_page.waiting_res = false;
                        login_page.toasts.error(message.clone());
                    }
                    _ => {}
                }
            }
        }

        if let Some(new_screen) = transition {
            self.screen = new_screen;
        }
    }
}
