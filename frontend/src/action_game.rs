use eframe::egui;

#[derive(Clone)]
pub enum StateAction {
	MOVE,
	// ATTACK,
}

pub struct ComandeButton {
	current_action: Option<StateAction>,
}

impl ComandeButton {
	pub fn macthing_action() -> Self {
		Self {current_action: None}
	}

	fn click_button(ui: &mut egui::Ui, label: &str) -> bool {
		ui.button(label).clicked()
	}

	pub fn draw_click_game(&mut self, ui: &mut egui::Ui, tx_outcomming: &std::sync::mpsc::Sender<String>) {
		let rect_screen = ui.max_rect();
		let pos_bottom = egui::pos2(rect_screen.center().x, rect_screen.max.y - (30.0));
		let rect_put = egui::Rect::from_center_size(pos_bottom, egui::Vec2::new(50.0, 40.0));

		match self.current_action.clone() {
			None => {
				ui.put(rect_put, |ui: &mut egui::Ui| {
					ui.horizontal(|ui| {
						if Self::click_button(ui, "MOVE") {
							self.current_action = Some(StateAction::MOVE);
						}
						// if Self::click_button(ui, "ATTACK") {
						// 	self.current_action = Some(StateAction::ATTACK);
						// }
					}).response
				});
			}
			Some(StateAction::MOVE) => {
					// egui::Panel::bottom("menu_direction")
					// .min_size(42.0_f32)
					// .show_inside(ui, |ui| {
					ui.put(rect_put, |ui: &mut egui::Ui| {
						ui.horizontal(|ui| {
						if Self::click_button(ui, "NORTH") {
							tx_outcomming.send("MOVE north".to_string()).unwrap();
							self.current_action = None;
						}
						if Self::click_button(ui, "SOUTH") {
							tx_outcomming.send("MOVE south".to_string()).unwrap();
							self.current_action = None;
						}
						if Self::click_button(ui, "EAST") {
							tx_outcomming.send("MOVE east".to_string()).unwrap();
							self.current_action = None;
						}
						if Self::click_button(ui, "OUEST") {
							tx_outcomming.send("MOVE ouest".to_string()).unwrap();
							self.current_action = None;
						}
					}).response
			});
		}
			// Some(StateAction::ATTACK) => {
			// 	tx_outcomming.send("ATTACK".to_string()).unwrap();
			// 	self.current_action = None;
			// };
		}
	}
}
