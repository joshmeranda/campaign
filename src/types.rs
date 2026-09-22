use serde::{Serialize, Deserialize};

macro_rules! proficiencies {
	($($iter:expr),* $(,)?) => {
		[
			$(
				$iter
			), *
		].iter().map(|s| String::from(*s)).collect()
	}
}

#[derive(Serialize, Deserialize)]
pub struct Spell {
	pub name: String,
	pub level: u8, // Level 0 denotes a Cantrip
	pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AbilityKind {
	Strength,
	Dexterity,
	Constitution,
	Intelligence,
	Wisdom,
	Charisma,
}

impl AbilityKind {
	pub fn proficiencies(&self) -> Vec<String> {
		match self {
			AbilityKind::Strength => proficiencies!("athletics"),
			AbilityKind::Dexterity => proficiencies!(""),
			AbilityKind::Constitution => proficiencies!(""),
			AbilityKind::Intelligence => proficiencies!("arcana", "history", "investigation", "nature", "religion"),
			AbilityKind::Wisdom => proficiencies!("animal handling", "insight", "medicine", "perception", "survival"),
			AbilityKind::Charisma => proficiencies!("deception", "intimidation", "performance", "persuassion"),
		}
	}
}

impl std::fmt::Display for AbilityKind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			AbilityKind::Strength => write!(f, "strength"),
			AbilityKind::Dexterity => write!(f, "dexterity"),
			AbilityKind::Constitution => write!(f, "constitution"),
			AbilityKind::Intelligence => write!(f, "intelligence"),
			AbilityKind::Wisdom => write!(f, "wisdom"),
			AbilityKind::Charisma => write!(f, "charisma"),
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct Ability {
	pub name: AbilityKind,
	pub score: u8,
	pub proficiencies: Vec<String>,
}

impl Ability {
	fn default_for_kind(k: AbilityKind) -> Ability {
		Ability {
			name: k,
			score: 0,
			proficiencies: k.proficiencies(),
		}
	}
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

impl Default for Character {
	fn default() -> Character {
		Character{
			name: String::new(),
			class: String::new(),
			race: String::new(),
			sub_class: String::new(),
			level: 0,
			armor_class: 0,
			max_hp: 0,
			speed: 0,
			abilities: [
				Ability::default_for_kind(AbilityKind::Strength),
				Ability::default_for_kind(AbilityKind::Dexterity),
				Ability::default_for_kind(AbilityKind::Constitution),
				Ability::default_for_kind(AbilityKind::Intelligence),
				Ability::default_for_kind(AbilityKind::Wisdom),
				Ability::default_for_kind(AbilityKind::Charisma),
			],
			spell_slots: [0, 0, 0, 0, 0, 0, 0, 0, 0],
			spells: vec![],
			rages: 0,
			languages: vec![],
			proficiencies: vec![],
		}
	}
}

impl Character {
	pub fn with_name(name: String) -> Character {
		Character{
			name,
			..Character::default()
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct Item {
	pub name: String,
	pub count: u8,
	pub description: String,
}

#[derive(Serialize, Deserialize, Default)]
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