mod window;
use window::MyTap;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
	let options_visualizeur = eframe::NativeOptions::default();

	eframe::run_native(
		"Answer Protocol",
		options_visualizeur,
		Box::new(|cc| {
			egui_extras::install_image_loaders(&cc.egui_ctx);
			Ok(Box::new(MyTap::default()))
		}),
	)
}
