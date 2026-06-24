use crate::parser::EventType;
use crate::parser::ServerMessage;
use serde_json;
use crate::{
    action_game::ComandeButton,
    room::state_mod::{GameScreen, StateRoom},
};
use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily, Ui};
use egui_notify::Toasts;
use std::sync::Arc;

pub struct MyTap {
    screen: Screen,
    pending_room: Option<StateRoom>,
    pub rx_incoming: std::sync::mpsc::Receiver<ServerMessage>,
    pub tx_outgoing: std::sync::mpsc::Sender<String>,
	chat_page: ChatPage,
    toasts: Toasts,
	state_exits: Vec<String>,
	items_room: Vec<String>,
	player_inventory: Vec<String>,
	server_logs: Vec<String>
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
            pending_room: None,
			state_exits: Vec::new(),
			items_room: Vec::new(),
			player_inventory: Vec::new(),
			server_logs: Vec::new()
		}
    }
}

// mult screen manager
enum Screen {
    LoginView(LoginPage),
    GameView(GameScreen),
    LoadingMod(u8),
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

struct SlashCommand {
    pattern: &'static str,
    protocol: &'static str,
    takes_arg: bool,
    hint: &'static str,
}

const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        pattern: "/group create",
        protocol: "GROUP CREATE",
        takes_arg: true,
        hint: "Create a group.",
    },
    SlashCommand {
        pattern: "/group invite",
        protocol: "GROUP INVITE",
        takes_arg: true,
        hint: "Invite someone in the current group.",
    },
    SlashCommand {
        pattern: "/group join",
        protocol: "GROUP JOIN",
        takes_arg: true,
        hint: "Join a group.",
    },
    SlashCommand {
        pattern: "/group leave",
        protocol: "GROUP LEAVE",
        takes_arg: true,
        hint: "Leave a group.",
    },
];

fn matching_commands(input: &str) -> Vec<&'static SlashCommand> {
    SLASH_COMMANDS
        .iter()
        .filter(|c| c.pattern.starts_with(input.trim_end()))
        .collect()
}

fn resolve_command(input: &str) -> Option<String> {
    for cmd in SLASH_COMMANDS {
        if cmd.takes_arg {
            if let Some(arg) = input.strip_prefix(&format!("{} ", cmd.pattern)) {
                return Some(format!("{} {}", cmd.protocol, arg.trim()));
            }
        } else if input.trim() == cmd.pattern {
            return Some(cmd.protocol.to_string());
        }
    }
    None
}

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
	fn show_logs(logs_serveur: &[String], ui: &mut egui::Ui) {
		for logs in logs_serveur {
			let color_log = if logs.starts_with("[ERR") {
				egui::Color32::from_rgb(210, 15, 57)
			}
			else if logs.starts_with("[Ok") {
				egui::Color32::from_rgb(166, 218, 169)
			}
			else if logs.starts_with("[EVT") {
				egui::Color32::from_rgb(23, 146, 153)
			}
			else {
				egui::Color32::WHITE
			};
			ui.label(egui::RichText::new(logs).size(11.0).color(color_log));
		}
	}

	fn valid_directions(server_reponse: &str) -> Vec<String> {
		let mut avaiable_pos = Vec::new();
		let lower_response = server_reponse.to_lowercase();

		if lower_response.contains("north") {
			avaiable_pos.push("north".to_string());
		}
		if lower_response.contains("south") {
			avaiable_pos.push("south".to_string());
		}
		if lower_response.contains("east") {
			avaiable_pos.push("east".to_string());
		}
		if lower_response.contains("west") {
			avaiable_pos.push("west".to_string());
		}
		return avaiable_pos; // add pos vec ex: north false south = ["south"]
	}

	fn parse_items(serveur_reponse: &str, look_key: &str) -> Vec<String> {
		let chunk_json = serveur_reponse.trim_start_matches("OK").trim();
		if let Ok(room) = serde_json::from_str::<serde_json::Value>(chunk_json) {
			if let Some(items) = room[look_key].as_array() {
				return items.iter()
				.filter_map(|i| i.as_str().map(String::from))
                .collect();
			}
		}
		Vec::new()
	}

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
                    if chat_page.message_input.starts_with('/') {
                        let suggestions = matching_commands(&chat_page.message_input);
                        if !suggestions.is_empty() {
                            ui.add_space(4.0);
                            egui::Frame::new()
                                .corner_radius(egui::CornerRadius::same(8_u8))
                                .inner_margin(egui::Margin::same(6))
                                .show(ui, |ui| {
                                    for cmd in suggestions {
                                        let label = format!("{}  —  {}", cmd.pattern, cmd.hint);
                                        if ui.selectable_label(false, label).clicked() {
                                            chat_page.message_input = if cmd.takes_arg {
                                                format!("{} ", cmd.pattern)
                                            } else {
                                                cmd.pattern.to_string()
                                            };
                                        }
                                    }
                                });
                        }
                    }
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
                            .id_salt("chat_input")
                            .hint_text("Type your message here...")
                            .font(egui::FontId::new(
                                20.0_f32,
                                egui::FontFamily::Name("undertale_font".into()),
                            )),
                    );

                    if res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !chat_page.message_input.trim().is_empty() {
                            if let Some(protocol_cmd) = resolve_command(&chat_page.message_input) {
                                tx.send(protocol_cmd).unwrap();
                            } else if chat_page.message_input.starts_with('/') {
                                // commande inconnue, idéalement un toast d'erreur ici
                                // (faudrait threader `toasts` jusqu'à draw_chat si tu veux ce feedback)
                            } else {
                                tx.send(format!(
                                    "CHAT {} {}",
                                    chat_page.scope, chat_page.message_input
                                ))
                                .unwrap();
                            }
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

					egui::Panel::bottom("logs")
						.min_size(150.0)
						.show_inside(ui, |ui| {
							ui.heading("Logs:");
							egui::ScrollArea::vertical()
						.auto_shrink([false, false])
						.stick_to_bottom(true)
						.show(ui, |ui| {
							Self::show_logs(&self.server_logs, ui);
						})
					});
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
                        let image_log_bg =
                            egui::include_image!("../asset_manager/login_page_v2.png");
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
                            .draw_click_game(ui, &self.tx_outgoing,
								&self.state_exits,
								&self.items_room,
								&self.player_inventory);
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

        if let Screen::GameView(_) = &mut self.screen {
            while let Ok(msg) = self.rx_incoming.try_recv() {
				match msg {
					// changement de salle (logique fichier 1)
                    ServerMessage::Ok(reponse) => {
						self.server_logs.push(format!("[Ok] {}", reponse));
						let valid_pos = Self::valid_directions(&reponse);
						if !valid_pos.is_empty() {
							self.state_exits = valid_pos;
						}

						let items_taken = Self::parse_items(&reponse, "items");
						self.items_room = items_taken;

						let item_inventory = Self::parse_items(&reponse, "inventory");
						self.player_inventory = item_inventory;

						let next_room_tr = if reponse.contains("loc.tavern") {
                            Some(StateRoom::Room1)
                        } else if reponse.contains("loc.square") {
                            Some(StateRoom::Room2)
                        } else if reponse.contains("loc.shop") {
                            Some(StateRoom::Room3)
                        } else if reponse.contains("loc.forest") {
                            Some(StateRoom::Room4)
                        } else if reponse.contains("loc.library") {
							Some(StateRoom::Room5)
						}
						else if reponse.contains("loc.observatory") {
							Some(StateRoom::Room6)
						}
						else if reponse.contains("loc.swamp") {
							Some(StateRoom::Room7)
						}
						else if reponse.contains("loc.crypt") {
							Some(StateRoom::Room8)
						}
						else {
                            None
                        };

						if let Some(room) = next_room_tr {
                            transition = Some(Screen::LoadingMod(60));
							self.pending_room = Some(room);
                        }
                        if reponse.contains("group=") {
                            self.toasts.success(format!("Group created: {}", reponse));
							self.tx_outgoing.send("LOOK".to_string()).unwrap();
						}
						if reponse.contains("taken=") {
							self.toasts.success(format!("taken {}", reponse));
							self.tx_outgoing.send("LOOK".to_string()).unwrap();
						}
						if reponse.contains("inventory=") {
							self.toasts.success(format!("inventory {}", reponse));
						}
						if reponse.contains("dropped=") {
							self.toasts.success(format!("dropped {}", reponse));
						}
                    }
                    // messages de chat (logique fichier 2) -> stockes dans self.chat_page
                    ServerMessage::Evt { evt_type, data } => match evt_type {
						// self.server_logs.push(format!("[OK] {} {}",evt_type, data));
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
                        EventType::GroupChat => {
                            let username = data.splitn(2, ' ').next().unwrap_or("").to_string();
                            let content = data.splitn(2, ' ').nth(1).unwrap_or("").to_string();
                            self.chat_page.messages.push(Message {
                                scope: Scope::Group,
                                username,
                                content,
                            });
                        }
                        EventType::Invite => {
                            self.toasts.info(format!("Group invitation: {}", data));
                        }
                        EventType::Join => {
                            self.toasts.info(format!("Someone join the group {}", data));
                        }
                        EventType::PresenceEnter => {
                            self.toasts.info(format!("{} enter the room", data));
                        }
                        EventType::PresenceLeave => {
                            self.toasts.info(format!("{} leave the room", data));
                        }
                        _ => {}
                    },

                    ServerMessage::Err { code, message } => {
						self.server_logs.push(format!("[ERR] {} {}", code, message));
						self.toasts.error(format!("Error {}: {}", code, message));
                    }
                }
            }
        }

        if let Some(new_screen) = transition {
            self.screen = new_screen;
        }
    }
}
