use eframe::egui;

pub fn draw(ui: &mut egui::Ui) {
	let room_asset = egui::include_image!("room_asset/ROOM_1_vF.jpg");
	let rect = ui.max_rect();

	egui::Image::new(room_asset).paint_at(ui, rect);
	// todo!("item");
}
