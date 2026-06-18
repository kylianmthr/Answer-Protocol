use eframe::egui;
use crate::action_game::ComandeButton;

use crate::room:: {
	room_1,
	room_2,
	room_3,
	room_4,
	room_5,
	room_6,
	room_7,
	room_8,
};

pub enum StateRoom {
	Room1,
	Room2,
	Room3,
	Room4,
	Room5,
	Room6,
	Room7,
	Room8,
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
			StateRoom::Room4 => room_4::draw(ui),
			StateRoom::Room5 => room_5::draw(ui),
			StateRoom::Room6 => room_6::draw(ui),
			StateRoom::Room7 => room_7::draw(ui),
			StateRoom::Room8 => room_8::draw(ui),
		}
	}
}


// Départ : Tu apparais dans Le Chaudron Baveur (loc.tavern).

// Tape south : Tu arrives sur la Place du Vieux Grimoire (loc.square). C'est le carrefour central de ton monde.

// Tape east : Tu visites la boutique Baguettes & Sortilèges (loc.shop).

// Tape west : Retour sur la Place (loc.square).

// Tape west : Tu entres dans la Bibliothèque Interdite (loc.library).

// Tape north : Tu montes dans la Tour de l'Astral (loc.observatory).

// Tape south : Retour dans la Bibliothèque (loc.library).

// Tape east : Retour sur la Place (loc.square).

// Tape south : Tu t'enfonces dans la Forêt des Murmures (loc.forest).

// Tape east : Tu découvres le Marais Putride (loc.swamp).

// Tape south : Tu atteins la dernière zone, la Crypte des Anciens (loc.crypt), où se trouve ton PNJ hostile (Malakor).