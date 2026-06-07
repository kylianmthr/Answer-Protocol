mod window;
use window::MyTap;
use eframe::egui;

fn main() {
	let options_visualizeur = eframe::NativeOptions::default();
	eframe::run_native(
		"Answer Protocol",
		options_visualizeur,
		Box::new(|_cc| Ok(Box::new(MyTap::default()))),
	);
}
