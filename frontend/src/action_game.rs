#[derive(Clone)]
pub enum StateAction {
	INVENTORY,
	ATTACK,
	MOVE,
	LOOK,
	TALK,
	TAKE,
	DROP,
	QUEST,
	QUIT,
}

pub struct ComandeButton {
	current_action: Option<StateAction>,
}

impl ComandeButton {
	pub fn macthing_action() -> Self {
		Self {current_action: None,
		}
	}

	fn click_button(ui: &mut egui::Ui, label: &str, good_pos: bool) -> bool {
		ui.add_enabled(
			good_pos,
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

	pub fn draw_click_game(&mut self, ui: &mut egui::Ui, tx_outcomming: &std::sync::mpsc::Sender<String>,
		avaiable_vecpos: &[String],
		state_items: &[String],
		state_inventory: &[String]) {
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
						if Self::click_button(ui, "MOVE", true) {
							self.current_action = Some(StateAction::MOVE);
						}
						if Self::click_button(ui, "ATTACK", true) {
							self.current_action = Some(StateAction::ATTACK);
						}
						if Self::click_button(ui, "LOOK", true) {
							self.current_action = Some(StateAction::LOOK);
						}
						if Self::click_button(ui, "TALK", true) {
							self.current_action = Some(StateAction::TALK);
						}
						if Self::click_button(ui, "INVENTORY", true) {
							self.current_action = Some(StateAction::INVENTORY);
						}
						if Self::click_button(ui, "TAKE", true) {
							self.current_action = Some(StateAction::TAKE);
						}
						if Self::click_button(ui, "DROP", true) {
							self.current_action = Some(StateAction::DROP);
						}
						if Self::click_button(ui, "QUEST", true) {
							self.current_action = Some(StateAction::QUEST);
						}
					}).response
				});
				ui.put(rect_put_top, |ui: &mut egui::Ui| {
					ui.horizontal(|ui| {
						if Self::click_button(ui, "QUIT", true) {
							self.current_action = Some(StateAction::QUIT);
						}
					}).response
				});
			}
			// sous munu 1
			Some(StateAction::MOVE) => {
				let go_north = avaiable_vecpos.contains(&"north".to_string());
				let go_south = avaiable_vecpos.contains(&"south".to_string());
				let go_east = avaiable_vecpos.contains(&"east".to_string());
				let go_west = avaiable_vecpos.contains(&"west".to_string());
				
				ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
						ui.horizontal(|ui| {
						if Self::click_button(ui, "NORTH", go_north) {
							tx_outcomming.send("MOVE north".to_string()).unwrap();
							self.current_action = None;
						}
						if Self::click_button(ui, "SOUTH", go_south) {
							tx_outcomming.send("MOVE south".to_string()).unwrap();
							self.current_action = None;
						}
						if Self::click_button(ui, "EAST", go_east) {
							tx_outcomming.send("MOVE east".to_string()).unwrap();
							self.current_action = None;
						}
						if Self::click_button(ui, "WEST", go_west) {
							tx_outcomming.send("MOVE west".to_string()).unwrap();
							self.current_action = None;
						}
						if Self::click_button(ui, "BACK", true) {
							self.current_action = None;
						}
					}).response
			});
		}
		Some(StateAction::INVENTORY) => {
			ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
				ui.horizontal(|ui| {
					for item_in in state_inventory {
						if Self::click_button(ui, item_in, true) {
							ui.label(item_in);
						}
					}
					if Self::click_button(ui, "BACK", true) {
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
		// sous menu
		Some(StateAction::TAKE) => {
			ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
				ui.horizontal(|ui| {
					let mut taken: bool = false;
						for item_id in state_items {
							if Self::click_button(ui, item_id, true) {
								tx_outcomming.send(format!("TAKE {}", item_id)).unwrap();
								taken = true;
							}
						}
					if taken {
						self.current_action = None;
					}
					if Self::click_button(ui, "BACK", true) {
						self.current_action = None;
					}
				}).response
			});
		}
		Some(StateAction::DROP) => {
			ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
				ui.horizontal(|ui| {
					let mut drop: bool = false;
					for item_id in state_items {
						if Self::click_button(ui, item_id, true) {
							tx_outcomming.send(format!("DROP {}", item_id)).unwrap();
							drop = true;
						}
					}
					if drop {
						self.current_action = None;
					}
					if Self::click_button(ui, "BACK", true) {
						self.current_action = None;
					}
				}).response
			});
		}
		Some(StateAction::QUEST) => {
			tx_outcomming.send("QUEST".to_string()).unwrap();
			self.current_action = None;
		}
		Some(StateAction::QUIT) => {
			tx_outcomming.send("QUIT".to_string()).unwrap();
			ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
		}
		}
	}
}
