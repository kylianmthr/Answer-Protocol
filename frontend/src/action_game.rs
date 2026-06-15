#[derive(Clone)]
pub enum StateAction {
	INVENTORY,
	ATTACK,
	MOVE,
	LOOK,
	TALK,
	QUIT,
}

pub struct ComandeButton {
	current_action: Option<StateAction>,
}

impl ComandeButton {
	pub fn macthing_action() -> Self {
		Self {current_action: None}
	}

	fn click_button(ui: &mut egui::Ui, label: &str) -> bool {
		ui.add(
			egui::Button::new(egui::RichText::new(label)
			.size(16.5_f32)
			.color(egui::Color32::from_rgb(205, 214, 244))
		)

		.min_size(egui::Vec2::new(95.0, 45.0))
			.corner_radius(egui::CornerRadius::same(18_u8))
			.fill(egui::Color32::from_rgb(49, 50, 68))
			.stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(114, 135, 253)))
		).clicked()
	}

	pub fn draw_click_game(&mut self, ui: &mut egui::Ui, tx_outcomming: &std::sync::mpsc::Sender<String>) {
		let rect_screen = ui.max_rect();
		let pos_bottom = egui::pos2(rect_screen.center().x, rect_screen.max.y - (60.0));
		let pos_top = egui::pos2(rect_screen.min.x + 10.0, rect_screen.min.y + 10.0);

		let rect_put_bottom = egui::Rect::from_center_size(pos_bottom, egui::Vec2::new(300.0, 40.0));
		let rect_put_top = egui::Rect::from_min_size(pos_top, egui::Vec2::new(100.0, 40.0));

		match self.current_action.clone() {
			None => {
				// menu princpipal
				ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
					ui.horizontal(|ui| {
						if Self::click_button(ui, "MOVE") {
							self.current_action = Some(StateAction::MOVE);
						}
						if Self::click_button(ui, "ATTACK") {
							self.current_action = Some(StateAction::ATTACK);
						}
						if Self::click_button(ui, "LOOK") {
							self.current_action = Some(StateAction::LOOK);
						}
						if Self::click_button(ui, "TALK") {
							self.current_action = Some(StateAction::TALK);
						}
						if Self::click_button(ui, "INVENTORY") {
							self.current_action = Some(StateAction::INVENTORY);
						}
					}).response
				});
				ui.put(rect_put_top, |ui: &mut egui::Ui| {
					ui.horizontal(|ui| {
						if Self::click_button(ui, "QUIT") {
							self.current_action = Some(StateAction::QUIT);
							// ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
						}
					}).response
				});
			}
			// sous munu 1
			Some(StateAction::MOVE) => {
					ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
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
						if Self::click_button(ui, "BACK") {
							self.current_action = None;
						}
					}).response
			});
		}
		Some(StateAction::INVENTORY) => {
			ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
				ui.horizontal(|ui| {
					ui.label("invenvtory comming soon ...");
					if Self::click_button(ui, "BACK") {
						self.current_action = None;
					}
				}).response
			});
		}
		Some(StateAction::ATTACK) => {
				tx_outcomming.send("ATTACK".to_string()).unwrap();
				self.current_action = None;
			}
		Some(StateAction::LOOK) => {
				tx_outcomming.send("LOOK".to_string()).unwrap();
				self.current_action = None;
			}
		Some(StateAction::TALK) => {
			tx_outcomming.send("TALK".to_string()).unwrap();
			self.current_action = None;
		}
		Some(StateAction::QUIT) => {
			tx_outcomming.send("QUIT".to_string()).unwrap();
			ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
		}
		}
	}
}
