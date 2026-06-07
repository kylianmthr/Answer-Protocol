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

// apply contrat (App) on MyTap
impl eframe::App for MyTap{
	// modify (mut) once per frame
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui|
		{
			ui.heading("Awnser Protocol");
			// add atrr
		});
	}
}