use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Spell {
	pub name: String,
	pub level: u8, // Level 0 denotes a Cantrip
	pub description: String,
}

// todo: this currently only supports proficiencies known at build time
pub type Proficiency = (String, bool);

#[derive(Serialize, Deserialize)]
pub struct Ability {
	pub name: String,
	pub score: u8,
	pub proficiencies: [Proficiency; 5],
}

#[derive(Serialize, Deserialize)]
pub struct Character {
	pub name: String,
	pub class: String,
	pub race: String,
	pub sub_class: String,
	pub level: u8, // proficiency is tied to level
	pub armor_class: u8,
	pub max_hp: u8,
	pub speed: u8,
	pub abilities: [Ability; 6],
	pub spell_slots: [u8; 9],
	pub spells: Vec<Spell>,
	pub rages: u8,
	pub languages: Vec<String>,
	pub proficiencies: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Item {
	pub name: String,
	pub count: u8,
	pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct Status{
	pub damage: u8,
	pub temp_hp: u8,
	pub used_slots: [u8; 9],
	pub used_rages: u8,

	pub luck: u8,

	pub coins: [u8; 5], // platinum gold electrum silver copper
	pub items: Vec<Item>,
}

impl Status {
	pub fn damage(&mut self, damage: u8) {
		let absorbed= self.temp_hp.min(damage);
		let actual_damage = damage.saturating_sub(absorbed);

		self.temp_hp -= absorbed;
		self.damage = self.damage.saturating_add(actual_damage)
	}

	pub fn heal(&mut self, heal: u8) {
		self.damage = self.damage.saturating_sub(heal);
	}

	// Decrements the amount of available spell slots are the given level. Slot is ignored if the given number is outside the valid range [0-9].
	pub fn cast(&mut self, slot: usize) {
		if slot >= 1 && slot <= 9 {
			self.used_slots[slot-1] += 1;
		}
	}

	pub fn rage(&mut self) {
		self.used_rages += 1
	}
}

#[cfg(test)]
mod test_status {
	use super::*;

	#[test]
	fn test_damage() {
		struct Test<'a> {
			title: &'a str,

			status: Status,
			damage: u8,

			expected: u8,
			expected_temp: u8,
		}

		let cases = vec![
			Test{
				title: "with full hp",
				status: Status{damage: 0, temp_hp: 0, used_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0], used_rages:0, luck: 0, coins: [0, 0, 0, 0, 0], items: vec![], },
				damage: 10,
				expected: 10,
				expected_temp: 0,
			},
			Test{
				title: "with full hp and temp hp",
				status: Status{damage: 0, temp_hp: 5, used_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0], used_rages:0, luck: 0, coins: [0, 0, 0, 0, 0], items: vec![], },
				damage: 10,
				expected: 5,
				expected_temp: 0,
			},
			Test{
				title: "with damage",
				status: Status{damage: 10, temp_hp: 0, used_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0], used_rages:0, luck: 0, coins: [0, 0, 0, 0, 0], items: vec![], },
				damage: 10,
				expected: 20,
				expected_temp: 0,
			},
			Test{
				title: "with damage and temp_hp",
				status: Status{damage: 10, temp_hp: 5, used_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0], used_rages:0, luck: 0, coins: [0, 0, 0, 0, 0], items: vec![], },
				damage: 10,
				expected: 15,
				expected_temp: 0,
			},
			Test{
				title: "small damage with damage and temp_hp",
				status: Status{damage: 10, temp_hp: 5, used_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0], used_rages:0, luck: 0, coins: [0, 0, 0, 0, 0], items: vec![], },
				damage: 1,
				expected: 10,
				expected_temp: 4,
			},
		];

		for c in cases {
			let mut status = c.status;
			status.damage(c.damage);

			assert!(c.expected == status.damage, "{}", c.title);
			assert!(c.expected_temp == status.temp_hp, "{}", c.title);
		}
	}

	#[test]
	fn test_heal() {
		struct Test<'a> {
			title: &'a str,
		
			status: Status,
			heal: u8,

			expected: u8,
		}

		let cases = vec![
			Test{
				title: "with no damage",
				status: Status{damage: 0, temp_hp: 0, used_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0], used_rages:0, luck: 0, coins: [0, 0, 0, 0, 0], items: vec![], },
				heal: 10,
				expected: 0,
			},
			Test{
				title: "with less damage",
				status: Status{damage: 5, temp_hp: 0, used_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0], used_rages:0, luck: 0, coins: [0, 0, 0, 0, 0], items: vec![], },
				heal: 10,
				expected: 0,
			},
			Test{
				title: "with more damage",
				status: Status{damage: 15, temp_hp: 0, used_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0], used_rages:0, luck: 0, coins: [0, 0, 0, 0, 0], items: vec![], },
				heal: 10,
				expected: 5,
			},
		];

		for c in cases {
			let mut status = c.status;
			status.heal(c.heal);

			assert!(c.expected == status.damage, "{}", c.title);
		}
	}
}