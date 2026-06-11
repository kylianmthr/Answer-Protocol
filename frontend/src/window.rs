use crate::auth::auth;
use crate::parser::ServerMessage;
use eframe::egui;
use egui::Ui;
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
    item: String,
}

impl MyTap {
    fn draw_field_log(ui: &mut egui::Ui, login_page: &mut LoginPage) {
        ui.vertical_centered(|ui| {
            ui.add_space(250.0);

            let style_field = ui.style_mut();

            style_field.visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;
            style_field.override_font_id = Some(egui::FontId::proportional(24.0));

            ui.colored_label(egui::Color32::WHITE, "username:");
            ui.text_edit_singleline(&mut login_page.username);

            ui.add_space(42.0);
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
    }
}

// apply contrat (App) on MyTap
impl eframe::App for MyTap {
    // modify (mut) once per frame
    fn ui(&mut self, ctx: &mut Ui, _frame: &mut eframe::Frame) {
        let remove_border_bg =
            egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(0));
        egui::CentralPanel::default()
            .frame(remove_border_bg)
            .show(ctx, |ui| {
                let image_log_bg = egui::include_image!("../asset_manager/log_bg_1.png");
                let get_rect_screen = ui.max_rect(); // window_size

                egui::Image::new(image_log_bg).paint_at(ui, get_rect_screen);
                match &mut self.screen {
                    Screen::LoginView(login_page) => {
                        Self::draw_field_log(ui, login_page);
                    }
                    Screen::GameView(game_page) => {
                        todo!("atrr of game screen") // c'est genial todo // oui mais je prefere les derives
                    }
                };
            });

        if let Screen::LoginView(login_page) = &mut self.screen {
            login_page.toasts.show(ctx);

            if login_page.waiting_res {
                match login_page.rx_incoming.try_recv() {
                    Ok(ServerMessage::Ok(_)) => {
                        login_page.waiting_res = false;
                        login_page.toasts.success("Login successful".to_string());
                    }
                    Ok(ServerMessage::Err { code: 500, message }) => {
                        login_page.waiting_res = false;
                        login_page.toasts.error(message.clone());
                    }
                    _ => {}
                }
            }
        }
    }
}
