use eframe::egui;


// init screen egui(GUI)
//app::MyTape
pub struct MyTap {
	screen:Screen,
}

// default start program into login page
// modify into update fn with self.screen to change state
//e.g self.screen = Screen::GameView{...}
impl Default for MyTap {
	fn default() -> Self {
		Self {
			screen:Screen::LoginView(LoginPage::default()),
		}
	}
}

// mult screen manager
enum Screen {
	LoginView(LoginPage),
	GameView(GamePage)
}

// macro to define default field with given type username == ""
#[derive(Default)]
struct LoginPage {
	username: String,
	serveur_adrr: String // sock into str -> .parse() convert to u16
}

struct GamePage {
	item: String
}

impl MyTap {
	fn draw_field_log(&mut self, ui: &mut egui::Ui, login_page: &mut LoginPage) {
		ui.vertical_centered(|ui| {
						ui.add_space(250.0);

						let style_field = ui.style_mut();

						style_field.visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;
						style_field.override_font_id = Some(
							egui::FontId::proportional(24.0));

						ui.colored_label(egui::Color32::WHITE, "username:");
						ui.text_edit_singleline(&mut login_page.username);

						ui.add_space(42.0);
						ui.colored_label(egui::Color32::WHITE, "address serveur:");
						ui.text_edit_singleline(&mut login_page.serveur_adrr);
		});
	}
}

// apply contrat (App) on MyTap
impl eframe::App for MyTap{
	// modify (mut) once per frame
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let remove_border_bg = egui::Frame::central_panel(&ctx.style()).
				inner_margin(egui::Margin::same(0));
		egui::CentralPanel::default()
		.frame(remove_border_bg)
		.show(ctx, |ui|
		{
			let image_log_bg = egui::include_image!("../asset_manager/log_bg_1.png");
			let get_rect_screen = ui.max_rect(); // window_size

			egui::Image::new(image_log_bg).paint_at(ui, get_rect_screen);
			match &mut self.screen {
				Screen::LoginView(login_page) => {

						self.draw_field_log(ui, login_page);
				}
				Screen::GameView(game_page) => {
					todo!("atrr of game screen")  // c'est genial todo
				}
			};
		});
	}
}
