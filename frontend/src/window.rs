use std::sync::Arc;

use crate::parser::ServerMessage;
use crate::{auth::auth, parser::EventType};
use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily, Ui};
use egui_notify::Toasts;

// init screen egui(GUI)
//app::MyTape
pub struct MyTap {
    screen: Screen,
    rx_incoming: std::sync::mpsc::Receiver<ServerMessage>,
    tx_outgoing: std::sync::mpsc::Sender<String>,
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
            screen: Screen::LoginView(LoginPage::default()),
            rx_incoming,
            tx_outgoing,
        }
    }
}

// mult screen manager
enum Screen {
    LoginView(LoginPage),
    GameView(GamePage),
}

// macro to define default field with given type username == ""
#[derive(Default)]
struct LoginPage {
    username: String,
    serveur_adrr: String, // sock into str -> .parse() convert to u16
    toasts: Toasts,
    waiting_res: bool,
}

struct GamePage {
    username: String,
    chat_page: ChatPage,
}

#[derive(Debug, Clone, PartialEq)]
enum Scope {
    Room,
    Group,
    Global,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Room => write!(f, "ROOM"),
            Scope::Group => write!(f, "GROUP"),
            Scope::Global => write!(f, "GLOBAL"),
        }
    }
}

struct Message {
    scope: Scope,
    username: String,
    content: String,
}

struct ChatPage {
    scope: Scope,
    messages: Vec<Message>,
    message_input: String,
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
    fn draw_field_log(
        ui: &mut egui::Ui,
        login_page: &mut LoginPage,
        tx: &std::sync::mpsc::Sender<String>,
    ) {
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
                    tx.send(format!("CONNECT {}", login_page.username)).unwrap();
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
            ui.label("test");
        });
    }

    fn draw_chat(
        ui: &mut egui::Ui,
        chat_page: &mut ChatPage,
        tx: &std::sync::mpsc::Sender<String>,
    ) {
        ui.vertical_centered(|ui| {
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

                let res = ui.add(
                    egui::TextEdit::singleline(&mut chat_page.message_input)
                        .hint_text("Type your message here...")
                        .font(egui::FontId::new(
                            20.0_f32,
                            egui::FontFamily::Name("undertale_font".into()),
                        )),
                );

                if res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !chat_page.message_input.trim().is_empty() {
                        // Send the message to the server
                        tx.send(format!(
                            "CHAT {} {}",
                            chat_page.scope, chat_page.message_input
                        ))
                        .unwrap();
                        chat_page.message_input.clear();
                    }
                }
            });
        });
    }

    fn draw_scope_button(ui: &mut egui::Ui, chat_page: &mut ChatPage) {
        ui.horizontal(|ui| {
            let scopes = [Scope::Room, Scope::Group, Scope::Global];
            for scope in &scopes {
                let is_selected = chat_page.scope == *scope;
                let button = ui.selectable_label(is_selected, scope.to_string());
                if button.clicked() {
                    chat_page.scope = scope.clone();
                }
            }
        });
    }
}

// apply contrat (App) on MyTap
impl eframe::App for MyTap {
    // modify (mut) once per frame
    fn ui(&mut self, ctx: &mut Ui, _frame: &mut eframe::Frame) {
        let remove_border_bg =
            egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(0));
        //if matches!(&self.screen, Screen::GameView(_)) {
        if let Screen::GameView(game_page) = &mut self.screen {
            egui::SidePanel::right("chat_panel")
                .min_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Chat");
                    egui::TopBottomPanel::bottom("chat_input").show_inside(ui, |ui| {
                        Self::draw_chat(ui, &mut game_page.chat_page, &self.tx_outgoing.clone());
                    });
                    egui::TopBottomPanel::top("scope_select").show_inside(ui, |ui| {
                        Self::draw_scope_button(ui, &mut game_page.chat_page);
                    });
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for msg in &game_page.chat_page.messages {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("[{}]", msg.scope));
                                        ui.label(format!("{}:", msg.username));
                                        ui.label(&msg.content);
                                    });
                                }
                            });
                    });
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
                        Self::draw_field_log(ui, login_page, &self.tx_outgoing.clone());
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
                match self.rx_incoming.try_recv() {
                    Ok(ServerMessage::Ok(_)) => {
                        login_page.waiting_res = false;
                        login_page.toasts.success("Login successful".to_string());
                        transition = Some(Screen::GameView(GamePage {
                            username: login_page.username.clone(),
                            chat_page: ChatPage {
                                scope: Scope::Room,
                                messages: Vec::new(),
                                message_input: String::new(),
                            },
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

        if let Screen::GameView(game_page) = &mut self.screen {
            while let Ok(msg) = self.rx_incoming.try_recv() {
                match msg {
                    ServerMessage::Evt { evt_type, data } => match evt_type {
                        EventType::RoomChat => {
                            let username = data.splitn(2, ' ').next().unwrap_or("").to_string();
                            let msg = data.splitn(2, ' ').nth(1).unwrap_or("").to_string();
                            game_page.chat_page.messages.push(Message {
                                scope: Scope::Room,
                                username: username,
                                content: msg,
                            });
                        }
                        EventType::GlobalChat => {
                            let username = data.splitn(2, ' ').next().unwrap_or("").to_string();
                            let msg = data.splitn(2, ' ').nth(1).unwrap_or("").to_string();
                            game_page.chat_page.messages.push(Message {
                                scope: Scope::Global,
                                username: username,
                                content: msg,
                            });
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        if let Some(new_screen) = transition {
            self.screen = new_screen;
        }
    }
}
