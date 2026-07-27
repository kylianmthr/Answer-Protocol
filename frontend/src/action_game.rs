#[derive(Clone)]
pub enum StateAction {
    Inventory,
    Attack,
    Move,
    Look,
    Talk,
    Quest,
    Take,
    Quit,
	Who,
	Status,
}

pub struct CommandButton {
    current_action: Option<StateAction>,
}

impl CommandButton {
    pub fn macthing_action() -> Self {
        Self {
            current_action: None,
        }
    }

    pub fn click_button(ui: &mut egui::Ui, label: &str, good_pos: bool) -> bool {
        ui.add_enabled(
            good_pos,
            egui::Button::new(
                egui::RichText::new(label)
                    .size(16.5_f32)
                    .color(egui::Color32::from_rgb(205, 214, 244)),
            )
            .min_size(egui::Vec2::new(95.0, 45.0))
            .corner_radius(egui::CornerRadius::same(18_u8))
            .fill(egui::Color32::from_rgb(49, 50, 68))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(114, 135, 253),
            )),
        )
        .clicked()
    }

    pub fn draw_click_game(
        &mut self,
        ui: &mut egui::Ui,
        tx_outcomming: &std::sync::mpsc::Sender<String>,
        avaiable_vecpos: &[String],
        available_items: &[String],
        available_npcs: &[String],
        inventory: &[String],
    ) {
        let rect_screen = ui.max_rect();
        let pos_bottom = egui::pos2(rect_screen.center().x, rect_screen.max.y - (60.0));
        let pos_top = egui::pos2(rect_screen.min.x + 10.0, rect_screen.min.y + 10.0);

        let rect_put_bottom =
            egui::Rect::from_center_size(pos_bottom, egui::Vec2::new(300.0, 40.0));
        let rect_put_top = egui::Rect::from_min_size(pos_top, egui::Vec2::new(100.0, 40.0));

        match self.current_action.clone() {
            None => {
                ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if Self::click_button(ui, "MOVE", true) {
                            self.current_action = Some(StateAction::Move);
                        }
                        if Self::click_button(ui, "ATTACK", true) {
                            self.current_action = Some(StateAction::Attack);
                        }
                        if Self::click_button(ui, "LOOK", true) {
                            self.current_action = Some(StateAction::Look);
                        }
                        if Self::click_button(ui, "TALK", true) {
                            self.current_action = Some(StateAction::Talk);
                        }
                        if Self::click_button(ui, "INVENTORY", true) {
                            tx_outcomming.send("INVENTORY".to_string()).unwrap();
                            self.current_action = Some(StateAction::Inventory);
                        }
                        if Self::click_button(ui, "QUEST", true) {
                            self.current_action = Some(StateAction::Quest);
                        }
                        if Self::click_button(ui, "TAKE", true) {
                            self.current_action = Some(StateAction::Take);
                        }
						if Self::click_button(ui, "WHO", true) {
							self.current_action = Some(StateAction::Who);
						}
						if Self::click_button(ui, "STATUS", true) {
							self.current_action = Some(StateAction::Status);
						}
                    })
                    .response
                });
                ui.put(rect_put_top, |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if Self::click_button(ui, "QUIT", true) {
                            self.current_action = Some(StateAction::Quit);
                        }
                    })
                    .response
                });
            }
            Some(StateAction::Move) => {
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
                    })
                    .response
                });
            }
            Some(StateAction::Inventory) => {
                ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if inventory.is_empty() {
                            ui.label("Inventory empty");
                        } else {
                            for item in inventory {
                                let label_str = item.split(".").last().unwrap_or(item);
                                if Self::click_button(ui, label_str, true) {
                                    tx_outcomming.send(format!("DROP {}", item)).unwrap();
                                }
                            }
                        }
                        if Self::click_button(ui, "BACK", true) {
                            self.current_action = None;
                        }
                    })
                    .response
                });
            }
            Some(StateAction::Attack) => {
                ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if available_npcs.is_empty() {
                            ui.label("No one to fight");
                        } else {
                            for npc in available_npcs {
                                let label_npc = npc.split(".").last().unwrap_or(npc);
                                if Self::click_button(ui, label_npc, true) {
                                    tx_outcomming.send(format!("ATTACK {}", npc)).unwrap();
                                }
                            }
                        }
                        if Self::click_button(ui, "STATUS", true) {
                            tx_outcomming.send("STATUS".to_string()).unwrap();
                        }
                        if Self::click_button(ui, "BACK", true) {
                            self.current_action = None;
                        }
                    })
                    .response
                });
            }
            Some(StateAction::Look) => {
                tx_outcomming.send("LOOK".to_string()).unwrap();
                self.current_action = None;
            }
			Some(StateAction::Who) => {
				tx_outcomming.send("WHO".to_string()).unwrap();
				self.current_action = None;
			}
			Some(StateAction::Status) => {
				tx_outcomming.send("STATUS".to_string()).unwrap();
				self.current_action = None;
			}
            Some(StateAction::Talk) => {
                ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if available_npcs.is_empty() {
                            ui.label("No one here");
                        } else {
                            for npc in available_npcs {
                                let label_npc = npc.split(".").last().unwrap_or(npc);
                                if Self::click_button(ui, npc, true) {
                                    tx_outcomming.send(format!("TALK {}", label_npc)).unwrap();
                                    self.current_action = None;
                                }
                            }
                        }
                        if Self::click_button(ui, "BACK", true) {
                            self.current_action = None;
                        }
                    })
                    .response
                });
            }
            Some(StateAction::Quest) => {
                ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if available_npcs.is_empty() {
                            ui.label("No one here");
                        } else {
                            for npc in available_npcs {
                                if Self::click_button(ui, npc, true) {
                                    tx_outcomming.send(format!("QUEST {}", npc)).unwrap();
                                    self.current_action = None;
                                }
                            }
                        }
                        if Self::click_button(ui, "SEE ACTIVE", true) {
                            tx_outcomming.send("QUESTS".to_string()).unwrap();
                            self.current_action = None;
                        }
                        if Self::click_button(ui, "BACK", true) {
                            self.current_action = None;
                        }
                    })
                    .response
                });
            }
            Some(StateAction::Take) => {
                ui.put(rect_put_bottom, |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if available_items.is_empty() {
                            ui.label("No items here");
                        } else {
                            for item in available_items {
                                let label_str = item.split(".").last().unwrap_or(item);
                                if Self::click_button(ui, label_str, true) {
                                    tx_outcomming.send(format!("TAKE {}", item)).unwrap();
                                }
                            }
                            if Self::click_button(ui, "TAKE ALL", true) {
                                for item in available_items {
                                    tx_outcomming.send(format!("TAKE {}", item)).unwrap();
                                }
                            }
                        }
                        if Self::click_button(ui, "BACK", true) {
                            self.current_action = None;
                        }
                    })
                    .response
                });
            }
            Some(StateAction::Quit) => {
                tx_outcomming.send("QUIT".to_string()).unwrap();
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}
