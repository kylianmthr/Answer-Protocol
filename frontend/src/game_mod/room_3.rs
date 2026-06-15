use eframe::egui;

pub fn draw(ui: &mut egui::Ui) {
	let room_asset = egui::include_image!("room_asset/ROOM_3_v2.png");
	let rect = ui.max_rect();

	egui::Image::new(room_asset).paint_at(ui, rect);
}
