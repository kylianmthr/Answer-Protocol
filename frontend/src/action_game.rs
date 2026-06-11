use eframe::egui;

pub fn click_game(ui: &mut egui::Ui, label: &str) -> bool {
	ui.button(label).clicked()
}
