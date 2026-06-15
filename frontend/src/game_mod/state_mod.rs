use eframe::egui;
use crate::action_game::ComandeButton;

use crate::game_mod:: {
	room_1,
	room_2,
	room_3,
};


pub enum StateRoom {
	Room1,
	Room2,
	Room3,
}


pub struct GameScreen {
	pub button_mod: ComandeButton,
	pub current_room: StateRoom,
}

impl GameScreen {
	pub fn draw_room(&mut self, ui: &mut egui::Ui) {
		match self.current_room {
			StateRoom::Room1 => room_1::draw(ui),
			StateRoom::Room2 => room_2::draw(ui),
			StateRoom::Room3 => room_3::draw(ui),
		}
	}
}
