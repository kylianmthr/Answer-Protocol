use crate::parser::EventType;
use crate::parser::ServerMessage;
use crate::{
    action_game::CommandButton,
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
    state_items: Vec<String>,
    state_npcs: Vec<String>,
    state_inventory: Vec<String>,
    server_logs: Vec<String>,
    pending_talk: bool,
    pending_group_leave: bool,
}

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
            state_items: Vec::new(),
            state_npcs: Vec::new(),
            state_inventory: Vec::new(),
            server_logs: Vec::new(),
            pending_talk: false,
            pending_group_leave: false,
        }
    }
}

enum Screen {
    LoginView(LoginPage),
    GameView(GameScreen),
    LoadingMod(u8),
    CombatView(CombatState),
    EndView(String),
}

struct CombatState {
    enemy: String,
    player_hp: i64,
    enemy_hp: i64,
    can_act: bool,
    enemy_turn_at: Option<f64>,
    last_msg: String,
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
    // show_panel_cmd: bool,
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
        takes_arg: false,
        hint: "Leave a group.",
    },
];

fn json_field(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find('"')? + start;
    Some(json[start..end].to_string())
}

fn json_array(json: &str, key: &str) -> Option<Vec<String>> {
    let pattern = format!("\"{}\":[", key);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find(']')? + start;
    Some(
        json[start..end]
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

fn json_object_keys(json: &str, key: &str) -> Option<Vec<String>> {
    let pattern = format!("\"{}\":{{", key);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find('}')? + start;
    Some(
        json[start..end]
            .split(',')
            .filter_map(|pair| {
                let k = pair.split(':').next()?.trim().trim_matches('"');
                if k.is_empty() {
                    None
                } else {
                    Some(k.to_string())
                }
            })
            .collect(),
    )
}

fn json_number(json: &str, key: &str) -> i64 {
    let pattern = format!("\"{}\":", key);
    match json.find(&pattern) {
        Some(start) => json[start + pattern.len()..]
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect::<String>()
            .parse()
            .unwrap_or(0),
        None => 0,
    }
}

fn parse_bare_array(json: &str) -> Vec<String> {
    json.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

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

    undertale_font
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "undertale_font".to_owned());

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

    fn draw_combat(
        ui: &mut egui::Ui,
        combat: &mut CombatState,
        tx: &std::sync::mpsc::Sender<String>,
    ) {
        let rect = ui.max_rect();
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(24, 18, 28));
        ui.vertical_centered(|ui| {
            ui.add_space(rect.height() / 2.0 - 150.0);
            ui.label(
                egui::RichText::new("⚔ COMBAT ⚔")
                    .size(40.0_f32)
                    .color(egui::Color32::from_rgb(243, 139, 168)),
            );
            ui.add_space(24.0);
            ui.label(
                egui::RichText::new(&combat.enemy)
                    .size(22.0_f32)
                    .color(egui::Color32::from_rgb(205, 214, 244)),
            );
            ui.label(
                egui::RichText::new(format!("Enemy HP: {}", combat.enemy_hp))
                    .size(18.0_f32)
                    .color(egui::Color32::from_rgb(243, 139, 168)),
            );
            ui.add_space(18.0);
            ui.label(
                egui::RichText::new(format!("Your HP: {}", combat.player_hp))
                    .size(18.0_f32)
                    .color(egui::Color32::from_rgb(166, 227, 161)),
            );
            ui.add_space(16.0);
            let turn_line = if combat.can_act {
                "Your turn".to_string()
            } else {
                combat.last_msg.clone()
            };
            ui.label(
                egui::RichText::new(turn_line)
                    .size(16.0_f32)
                    .color(egui::Color32::from_rgb(180, 190, 210)),
            );
            ui.add_space(24.0);
            if CommandButton::click_button(ui, "ATTACK", combat.can_act) {
                tx.send(format!("ATTACK {}", combat.enemy)).unwrap();
                combat.can_act = false;
                combat.last_msg = "You strike...".to_string();
            }
        });
    }

    fn draw_end(ui: &mut egui::Ui, enemy: &str, tx: &std::sync::mpsc::Sender<String>) {
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
        ui.vertical_centered(|ui| {
            ui.add_space(rect.height() / 2.0 - 80.0);
            ui.label(
                egui::RichText::new("VICTORY")
                    .size(48.0_f32)
                    .color(egui::Color32::from_rgb(114, 135, 253)),
            );
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(format!("You defeated {}", enemy))
                    .size(20.0_f32)
                    .color(egui::Color32::from_rgb(205, 214, 244)),
            );
            ui.add_space(40.0);
            if CommandButton::click_button(ui, "QUIT", true) {
                tx.send("QUIT".to_string()).unwrap();
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
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
        pending_group_leave: &mut bool,
    ) {
        // ui.vertical_centered(|ui| {
        //     ui.scope(|ui| {
        //         let style_field = ui.style_mut();
        //         let rounding_field = egui::CornerRadius::same(10_u8);

        //         style_field.visuals.extreme_bg_color = egui::Color32::WHITE;
        //         style_field.visuals.override_text_color = Some(egui::Color32::BLACK);

        //         style_field.visuals.widgets.active.corner_radius = rounding_field;
        //         style_field.visuals.widgets.hovered.corner_radius = rounding_field;
        //         style_field.visuals.widgets.inactive.corner_radius = rounding_field;
        //         style_field.override_font_id = Some(egui::FontId::proportional(24.0_f32));
        //         style_field.visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;

        // 		let res = ui.add(
        //             egui::TextEdit::singleline(&mut chat_page.message_input)
        //                 .hint_text("Type your message here...")
        //                 .font(egui::FontId::new(
        //                     20.0_f32,
        //                     egui::FontFamily::Name("undertale_font".into()),
        //                 )),
        //         );

        //         if res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        //             if !chat_page.message_input.trim().is_empty() {
        //                 // Send the message to the server
        //                 tx.send(format!(
        //                     "CHAT {} {}",
        //                     chat_page.scope, chat_page.message_input
        //                 ))
        //                 .unwrap();
        //                 chat_page.message_input.clear();
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
                            if protocol_cmd == "GROUP LEAVE" {
                                *pending_group_leave = true;
                            }
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

impl MyTap {
    fn show_logs(logs_serveur: &[String], ui: &mut egui::Ui) {
        for logs in logs_serveur {
            let color_log = if logs.starts_with("[ERR") {
                egui::Color32::from_rgb(210, 15, 57)
            } else if logs.starts_with("[Ok") {
                egui::Color32::from_rgb(166, 218, 169)
            } else if logs.starts_with("[EVT") {
                egui::Color32::from_rgb(23, 146, 153)
            } else {
                egui::Color32::WHITE
            };
            ui.label(egui::RichText::new(logs).size(11.0).color(color_log));
        }
    }
}

impl eframe::App for MyTap {
    fn ui(&mut self, ctx: &mut Ui, _frame: &mut eframe::Frame) {
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
                        Self::draw_chat(ui, &mut self.chat_page, &tx, &mut self.pending_group_leave);
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
                let get_rect_screen = ui.max_rect();
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
                        ui.ctx().request_repaint();
                    }
                    Screen::GameView(game_screen) => {
                        game_screen.draw_room(ui);
                        game_screen.button_mod.draw_click_game(
                            ui,
                            &self.tx_outgoing,
                            &self.state_exits,
                            &self.state_items,
                            &self.state_npcs,
                            &self.state_inventory,
                            &mut self.pending_talk,
                        );
                    }
                    Screen::CombatView(combat) => {
                        Self::draw_combat(ui, combat, &self.tx_outgoing);
                    }
                    Screen::EndView(enemy) => {
                        Self::draw_end(ui, enemy, &self.tx_outgoing);
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
                    Ok(ServerMessage::Err { code, message }) => {
                        self.toasts.error(format!("{} {}", code, message));
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
                    button_mod: CommandButton::macthing_action(),
                }));
            }
        }

        if let Screen::GameView(_) = &mut self.screen {
            while let Ok(msg) = self.rx_incoming.try_recv() {
                match msg {
                    ServerMessage::Ok(reponse) => {
                        self.server_logs.push(format!("[Ok] {}", reponse));
                        if self.pending_talk {
                            self.pending_talk = false;
                            if !reponse.is_empty() {
                                self.toasts.info(reponse.clone());
                            }
                            continue;
                        }
                        if self.pending_group_leave {
                            self.pending_group_leave = false;
                            self.toasts.info("You left the group".to_string());
                            continue;
                        }
                        if reponse.starts_with("room=") {
                            self.tx_outgoing.send("LOOK".to_string()).unwrap();
                        }
                        if let Some(exits) = json_object_keys(&reponse, "exits") {
                            self.state_exits = exits;
                        }

                        if let Some(items) = json_array(&reponse, "items") {
                            self.state_items = items;
                        }
                        if let Some(item_id) = reponse.strip_prefix("taken=") {
                            self.state_items.retain(|i| i != item_id);
                            self.state_inventory.push(item_id.to_string());
                            self.toasts.success(format!("Took {}", item_id));
                        }
                        if let Some(item_id) = reponse.strip_prefix("dropped=") {
                            self.state_inventory.retain(|i| i != item_id);
                            self.state_items.push(item_id.to_string());
                            self.toasts.success(format!("Dropped {}", item_id));
                        }
                        if reponse.starts_with('[') && reponse.contains("\"item.") {
                            self.state_inventory = parse_bare_array(&reponse);
                        }
                        if let Some(npcs) = json_array(&reponse, "npcs") {
                            self.state_npcs = npcs;
                        }

                        let next_room_tr = match json_field(&reponse, "id").as_deref() {
                            Some("loc.tavern") => Some(StateRoom::Room1),
                            Some("loc.square") => Some(StateRoom::Room2),
                            Some("loc.shop") => Some(StateRoom::Room3),
                            Some("loc.forest") => Some(StateRoom::Room4),
                            Some("loc.library") => Some(StateRoom::Room5),
                            Some("loc.observatory") => Some(StateRoom::Room6),
                            Some("loc.swamp") => Some(StateRoom::Room7),
                            Some("loc.crypt") => Some(StateRoom::Room8),
                            _ => None,
                        };

                        if let Some(room) = next_room_tr {
                            transition = Some(Screen::LoadingMod(90));
                            self.pending_room = Some(room);
                        }
                        if reponse.contains("group=") {
                            self.toasts.success(format!("Group created: {}", reponse));
                        }
                        if reponse.starts_with('{') && reponse.contains("\"quest_id\"") {
                            let name = json_field(&reponse, "name").unwrap_or_default();
                            let description =
                                json_field(&reponse, "description").unwrap_or_default();
                            let status = json_field(&reponse, "status").unwrap_or_default();
                            if status == "completed" {
                                let reward = json_field(&reponse, "reward").unwrap_or_default();
                                self.toasts.success(format!(
                                    "Quest completed: {} (reward: {})",
                                    name, reward
                                ));
                            } else {
                                self.toasts
                                    .info(format!("Quest: {} — {}", name, description));
                            }
                        }
                        if reponse.starts_with('[') && reponse.contains("\"quest_id\"") {
                            for entry in reponse.split('{').skip(1) {
                                let quest_id = json_field(entry, "quest_id").unwrap_or_default();
                                let status = json_field(entry, "status").unwrap_or_default();
                                let progress = json_field(entry, "progress").unwrap_or_default();
                                self.toasts
                                    .info(format!("{} — {} ({})", quest_id, status, progress));
                            }
                        }
                        if reponse.contains("\"max_hp\"") {
                            let hp = json_number(&reponse, "hp");
                            let max_hp = json_number(&reponse, "max_hp");
                            let label = json_field(&reponse, "status").unwrap_or_default();
                            self.toasts
                                .info(format!("HP: {}/{} ({})", hp, max_hp, label));
                        }
                        if reponse.contains("\"attacker_hp\"") {
                            let attacker_hp = json_number(&reponse, "attacker_hp");
                            let target_hp = json_number(&reponse, "target_hp");
                            let status = json_field(&reponse, "status").unwrap_or_default();
                            let enemy = json_field(&reponse, "enemy")
                                .or_else(|| self.state_npcs.first().cloned())
                                .unwrap_or_default();
                            match status.as_str() {
                                "victory" => transition = Some(Screen::EndView(enemy)),
                                "defeat" => {
                                    self.toasts.error(
                                        "You died! Back to the start — you can retry.".to_string(),
                                    );
                                    self.tx_outgoing.send("LOOK".to_string()).unwrap();
                                    self.pending_room = Some(StateRoom::Room1);
                                    transition = Some(Screen::LoadingMod(90));
                                }
                                _ => {
                                    let now = ctx.input(|i| i.time);
                                    let dmg = json_number(&reponse, "damage");
                                    transition = Some(Screen::CombatView(CombatState {
                                        enemy,
                                        player_hp: attacker_hp,
                                        enemy_hp: target_hp,
                                        can_act: false,
                                        enemy_turn_at: Some(now + 0.7),
                                        last_msg: format!("You hit for {}!", dmg),
                                    }));
                                }
                            }
                        }
                    }
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
                        EventType::Leave => {
                            self.toasts.info(format!("{} left the group", data));
                        }
                        EventType::PresenceEnter => {
                            self.toasts.info(format!("{} enter the room", data));
                        }
                        EventType::PresenceLeave => {
                            self.toasts.info(format!("{} leave the room", data));
                        }
                        EventType::Combat => {
                            self.toasts.info(format!("Combat: {}", data));
                        }
                        _ => {}
                    },

                    ServerMessage::Err { code, message } => {
                        self.pending_talk = false;
                        self.pending_group_leave = false;
                        self.toasts.error(format!("Error {}: {}", code, message));
                    }
                }
            }
        }

        if let Screen::CombatView(_) = &self.screen {
            let now = ctx.input(|i| i.time);
            if let Screen::CombatView(c) = &mut self.screen {
                if let Some(at) = c.enemy_turn_at {
                    if now >= at {
                        c.enemy_turn_at = None;
                        self.tx_outgoing
                            .send(format!("ATTACK {}", c.enemy))
                            .unwrap();
                    } else {
                        ctx.ctx().request_repaint();
                    }
                }
            }
            while let Ok(msg) = self.rx_incoming.try_recv() {
                match msg {
                    ServerMessage::Ok(reponse) if reponse.contains("\"attacker_hp\"") => {
                        let attacker_hp = json_number(&reponse, "attacker_hp");
                        let target_hp = json_number(&reponse, "target_hp");
                        let dmg = json_number(&reponse, "damage");
                        let actor = json_field(&reponse, "actor").unwrap_or_default();
                        let status = json_field(&reponse, "status").unwrap_or_default();
                        match status.as_str() {
                            "victory" => {
                                let enemy = match &self.screen {
                                    Screen::CombatView(c) => c.enemy.clone(),
                                    _ => String::new(),
                                };
                                transition = Some(Screen::EndView(enemy));
                            }
                            "defeat" => {
                                self.toasts.error(
                                    "You died! Back to the start — you can retry.".to_string(),
                                );
                                self.tx_outgoing.send("LOOK".to_string()).unwrap();
                                self.pending_room = Some(StateRoom::Room1);
                                transition = Some(Screen::LoadingMod(90));
                            }
                            _ => {
                                if let Screen::CombatView(c) = &mut self.screen {
                                    if actor == "enemy" {
                                        c.player_hp = attacker_hp;
                                        c.can_act = true;
                                        c.last_msg = format!("Enemy hits for {}!", dmg);
                                    } else {
                                        c.enemy_hp = target_hp;
                                        c.can_act = false;
                                        c.enemy_turn_at = Some(now + 0.7);
                                        c.last_msg = format!("You hit for {}!", dmg);
                                    }
                                }
                            }
                        }
                    }
                    ServerMessage::Err { code, message } => {
                        self.pending_talk = false;
                        self.pending_group_leave = false;
                        self.toasts.error(format!("Error {}: {}", code, message));
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
