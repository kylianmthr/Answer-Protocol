use crate::parser::EventType;
use crate::parser::ServerMessage;
use crate::{
    action_game::ComandeButton,
    game_mod::state_mod::{GameScreen, StateRoom},
};
use egui::epaint::color;
use egui::{FontData, FontDefinitions, FontFamily, Ui};
use egui_notify::Toasts;
use std::sync::Arc;
use eframe::egui;

pub struct MyTap {
	screen: Screen,
	pending_room: Option<StateRoom>,
    pub rx_incoming: std::sync::mpsc::Receiver<ServerMessage>,
    pub tx_outgoing: std::sync::mpsc::Sender<String>,
    chat_page: ChatPage,
	toasts: Toasts,
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
            screen: Screen::LoginView(LoginPage::new()),
            rx_incoming,
            tx_outgoing,
            toasts: Toasts::default(),
			chat_page: ChatPage::default(),
			pending_room: None
		}
    }
}

// mult screen manager
enum Screen {
    LoginView(LoginPage),
    LoadingMod(u8),
    GameView(GameScreen),
}

#[derive(Default)]
struct LoginPage {
    username: String,
    waiting_res: bool,
}

impl LoginPage {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            waiting_res: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
enum Scope {
    #[default]
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

#[derive(Default)]
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
    fn loading_animate(ui: &mut egui::Ui) {
        let get_rect = ui.max_rect();
        ui.painter()
            .rect_filled(get_rect, 0.0, egui::Color32::BLACK);

        let time_load = ui.ctx().input(|i| i.time);
        let a = ((time_load * 2.0).sin() * 127.0 + 128.0) as u8;

        let char_spining = ['*', ' '];
        let pos = (time_load * 1.0) as usize % char_spining.len();
        let spin = char_spining[pos];
        ui.vertical_centered(|ui| {
            ui.add_space(get_rect.height() / 2.0 - 30.0);
            ui.label(
                egui::RichText::new(format!(" {} UNDER_TAP", spin))
                    .size(32.0_f32)
                    .color(egui::Color32::from_rgba_unmultiplied(114, 125, 253, a)),
            );
        });
    }

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

                style_field.override_font_id = Some(egui::FontId::proportional(24.0_f32));
                style_field.visuals.override_text_color = Some(egui::Color32::BLACK);
                style_field.visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;
                style_field.visuals.widgets.inactive.corner_radius = rounding_field;
                style_field.visuals.widgets.hovered.corner_radius = rounding_field;
                style_field.visuals.widgets.active.corner_radius = rounding_field;
                style_field.visuals.extreme_bg_color = egui::Color32::WHITE;

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
			ui.visuals_mut().selection.bg_fill = egui::Color32::from_rgb(114, 135, 253);
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
        // toats log permanent
		self.toasts.show(ctx);
		if matches!(self.screen, Screen::GameView(_)) {
            let tx = self.tx_outgoing.clone();
            egui::Panel::right("chat_panel")
                .min_size(300.0)
				.show_inside(ctx, |ui| {
					ui.heading("Chat");

					egui::Panel::bottom("chat_input").show_inside(ui, |ui| {
                        Self::draw_chat(ui, &mut self.chat_page, &tx);
                    });

					egui::Panel::top("scope_select").show_inside(ui, |ui| {
                        Self::draw_scope_button(ui, &mut self.chat_page);
                    });

					egui::CentralPanel::default().show_inside(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for msg in &self.chat_page.messages {
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

        let remove_border_bg =
            egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(0));
        egui::CentralPanel::default()
            .frame(remove_border_bg)
            .show_inside(ctx, |ui| {
                let get_rect_screen = ui.max_rect(); // window_size
                match &mut self.screen {
                    Screen::LoginView(login_page) => {
                        let image_log_bg = egui::include_image!("../asset_manager/asset_up.jpeg");
                        egui::Image::new(image_log_bg).paint_at(ui, get_rect_screen);
                        Self::draw_field_log(ui, login_page, &self.tx_outgoing.clone());
                    }
                    Screen::LoadingMod(load_mod) => {
                        Self::loading_animate(ui);
                        *load_mod -= 1;
                        ui.ctx().request_repaint(); // frame / frame
                    }
                    Screen::GameView(game_screen) => {
                        game_screen.draw_room(ui);
                        game_screen
                            .button_mod
                            .draw_click_game(ui, &self.tx_outgoing);
                    }
                };
            });

        let mut transition: Option<Screen> = None;

		if let Screen::LoginView(login_page) = &mut self.screen {
            if login_page.waiting_res {
                match self.rx_incoming.try_recv() {
                    Ok(ServerMessage::Ok(_)) => {
                        login_page.waiting_res = false;
                        self.toasts.success("Login successful".to_string());
                        self.tx_outgoing.send("LOOK".to_string()).unwrap();
                        transition = Some(Screen::LoadingMod(90));
                    }
                    Ok(ServerMessage::Err { code: 500, message }) => {
                        self.toasts.error(message);
                        login_page.waiting_res = false;
                    }
                    _ => {}
                }
            }
        }

        if let Screen::LoadingMod(load_mod) = &mut self.screen {
            if *load_mod == 0 {
                let room = self.pending_room.take().unwrap_or(StateRoom::Room1);
				transition = Some(Screen::GameView(GameScreen {
                    current_room: room,
                    button_mod: ComandeButton::macthing_action(),
				}));
            }
        }

        if let Screen::GameView(game_screen) = &mut self.screen {
            while let Ok(msg) = self.rx_incoming.try_recv() {
                match msg {
                    // changement de salle (logique fichier 1)
                    ServerMessage::Ok(reponse) => {
                        let next_room_tr = if reponse.contains("loc.tavern") {
                            Some(StateRoom::Room1)
                        } else if reponse.contains("loc.square") {
                            Some(StateRoom::Room2)
						}
						else {
							None
						};
						if let Some(room) = next_room_tr {
							transition = Some(Screen::LoadingMod(90));
							self.pending_room = Some(room);
						}
                    }
                    // messages de chat (logique fichier 2) -> stockes dans self.chat_page
                    ServerMessage::Evt { evt_type, data } => match evt_type {
                        EventType::RoomChat => {
                            let username = data.splitn(2, ' ').next().unwrap_or("").to_string();
                            let content = data.splitn(2, ' ').nth(1).unwrap_or("").to_string();
                            self.chat_page.messages.push(Message {
                                scope: Scope::Room,
                                username,
                                content,
                            });
                        }
                        EventType::GlobalChat => {
                            let username = data.splitn(2, ' ').next().unwrap_or("").to_string();
                            let content = data.splitn(2, ' ').nth(1).unwrap_or("").to_string();
                            self.chat_page.messages.push(Message {
                                scope: Scope::Global,
                                username,
                                content,
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
