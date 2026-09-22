use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, Frame, layout};
use ratatui::widgets::{Block, Clear, Gauge, List, ListState, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState, Tabs, Wrap};
use ratatui::layout::{Alignment, Layout, Margin, Offset, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::layout::Constraint::{Length, Fill, Percentage};
use ratatui::text::{Line, Span, Text};
use ratatui::style::palette::tailwind::{SLATE, RED};
use utils::{color_for_damage_percent_lost};
use std::fs;
use serde::de::DeserializeOwned;

use crate::character::utils::to_roman_numerals;
use crate::types::{Ability, Character, Status};
use crate::error::AppError;

mod utils {
	use ratatui::style::Color;

	pub fn color_for_damage_percent_lost(percent_lost: f32) -> Color {
		if percent_lost >= 0.90 {
			Color::Red
		} else if percent_lost >= 0.75 {
			Color::Yellow
		} else if percent_lost >= 0.50 {
			Color::LightYellow
		} else if percent_lost != 0.0 {
			Color::Green
		} else {
			Color::LightGreen
		}
	}

	// Provides a limited and naive support for converting a number to a roman numeral. Only supports 0 < n < 10
	pub fn to_roman_numerals(n: usize) -> &'static str {
		match n {
			1 => "I",
			2 => "II",
			3 => "III",
			4 => "IV",
			5 => "V",
			6 => "VI",
			7 => "VII",
			8 => "VIII",
			9 => "IX",
			_ => panic!("number not supported, must be 0 < n < 10: {}", n)
		}
	}
}

#[derive(PartialEq, Copy, Clone)]
enum InputType {
	Damage,
	Heal,
	Cast,
}

#[derive(PartialEq, Copy, Clone)]
enum Tab {
	Items,
	Spells,
	Notes,
}

#[derive(PartialEq)]
enum AppMode {
	Idle,
	Help,
	Input(InputType),

	EditSelection,
	Editing,

	Error,
	Exiting,
}

impl Default for AppMode {
	fn default() -> Self {
		Self::Idle
	}
}

pub struct App {
	character_path: std::path::PathBuf,
	status_path: std::path::PathBuf,
	notes_path: std::path::PathBuf,

	mode: AppMode,

	input: String,
	head: usize,

	character: Character,
	status: Status,

	item_table_state: TableState,
	spell_table_state: TableState,
	edit_select_list_state: ListState,
	note_scrollbar_state: ScrollbarState,

	active_tab: Tab,

	state_updated: bool,

	// todo: provide a way to de-dup errors to prevent locking the UI (like when notes file does not exist or could not be read)
	err: Option<AppError>,
}

impl App {
	pub fn new(
		character_path: std::path::PathBuf,
		status_path: std::path::PathBuf,
		notes_path: std::path::PathBuf,
	) -> Result<Self, AppError> {
		let c: Character = Self::load_from_path(&character_path)?;
		let s: Status = Self::load_from_path(&status_path)?;

		Ok(App {
			character_path,
			status_path,
			notes_path,

			mode: AppMode::default(),

			input: String::new(),
			head: 0,

			character: c,
			status: s,

			item_table_state: TableState::new(),
			spell_table_state: TableState::new(),
			edit_select_list_state: ListState::default().with_selected(Some(0)),
			note_scrollbar_state: ScrollbarState::default(),

			active_tab: Tab::Items,

			state_updated: false,

			err: None,
		})
	}

	pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), AppError> {
		self.item_table_state.select_first();
		self.item_table_state.select_first_column();

		self.spell_table_state.select_first();
		self.spell_table_state.select_first_column();

		while self.mode != AppMode::Exiting {
			terminal.draw(|frame| {
				match self.mode {
					AppMode::Help => self.render_help(frame),
					_ => if let Err(err) = self.render(frame) {
						self.set_err(err);
					},
				}
			})?;

			match self.mode {
				AppMode::Idle => self.handle_events()?,
				AppMode::Help => self.handle_help_events()?,
				AppMode::Input(_) => self.handle_input_events()?,
				AppMode::EditSelection => self.handle_edit_select_events()?,
				AppMode::Editing => {
					if let Err(err) = self.edit_file() {
						self.set_err(err);
					} else {
						self.mode = AppMode::Idle;
					}

					terminal.clear()?;
				},
				AppMode::Error => _ = {
					event::read()?; // we don't care about the actual key-press here
					self.mode = AppMode::Idle;
				},
				AppMode::Exiting => terminal.clear()?,
			};

			if self.state_updated {
				let s = serde_yaml::to_string(&self.status)?;
				fs::write(&self.status_path, s)?;
			}
		}

		Ok(())
	}

	fn load_from_path<P, T>(path: P) -> Result<T, AppError>
	where
		P: AsRef<std::path::Path>,
		T: DeserializeOwned,
	{
		let data = fs::read_to_string(path)?;
		Ok(serde_yaml::from_str(&data)?)
	}

	fn edit_file(&mut self) -> Result<(), AppError> {
		if self.edit_select_list_state.selected().is_none() {
			return Ok(())
		}

		let selected = self.edit_select_list_state.selected().unwrap();

		let path = match selected {
			0 => self.character_path.clone(),
			1 => self.status_path.clone(),
			2 => self.notes_path.clone(),
			_ => panic!("selection should never be greate than 2"),
		};
		
		let path_name = path.to_str().unwrap();

		_ = std::process::Command::new("nano")
			.args(["--autoindent", path_name])
			.status()?;

		match selected {
			0 => self.character = Self::load_from_path(path)?,
			1 => self.status = Self::load_from_path(path)?,
			_ => {},
		}

		Ok(())
	}

	fn set_err(&mut self, err: AppError) {
		self.err = Some(err);
		self.mode = AppMode::Error;
	}

	fn handle_events(&mut self) -> Result<(), AppError> {
		if let Some(key) = event::read()?.as_key_press_event() {
			match key.code {
				KeyCode::Char('q') => {
					if self.mode == AppMode::Idle {
						self.mode = AppMode::Exiting
					}
				},
				KeyCode::Char('d') => self.mode = AppMode::Input(InputType::Damage),
				KeyCode::Char('h') => self.mode = AppMode::Input(InputType::Heal),
				KeyCode::Char('r') => {
					if self.status.used_rages == self.character.rages {
						self.set_err(AppError::Error(String::from("You ran out of rages")))
					} else {
						self.status.rage();
						self.state_updated = true;
					}
				},
				KeyCode::Char('e') => self.mode = AppMode::EditSelection,
				KeyCode::Char('c') => self.mode = AppMode::Input(InputType::Cast),
				KeyCode::Char('?') => self.mode = AppMode::Help,

				KeyCode::Up => match self.active_tab {
					Tab::Items => self.item_table_state.select_previous(),
					Tab::Spells => self.spell_table_state.select_previous(),
					Tab::Notes => self.note_scrollbar_state.scroll(ratatui::widgets::ScrollDirection::Backward),
				},
				KeyCode::Down =>  match self.active_tab {
					Tab::Items => self.item_table_state.select_next(),
					Tab::Spells => self.spell_table_state.select_next(),
					Tab::Notes => self.note_scrollbar_state.scroll(ratatui::widgets::ScrollDirection::Forward),
				},

				KeyCode::Right => self.active_tab = match self.active_tab {
					Tab::Items => if self.character.spells.len() > 0 {
						Tab::Spells
					} else {
						Tab::Notes
					},
					Tab::Spells => Tab::Notes,
					Tab::Notes => Tab::Items,
				},
				KeyCode::Left => self.active_tab = match self.active_tab {
					Tab::Items => Tab::Notes,
					Tab::Spells => Tab::Items,
					Tab::Notes => if self.character.spells.len() > 0 {
						Tab::Spells
					} else {
						Tab::Items
					}
				},

				_ => {}
			}
		}

		Ok(())
	}

	fn handle_help_events(&mut self) -> Result<(), AppError> {
		if let Some(key) = event::read()?.as_key_press_event() {
			match key.code {
				KeyCode::Esc => self.mode = AppMode::Idle,
				_ => {}
			}
		}

		Ok(())
	}

	fn handle_input_events(&mut self) -> Result<(), AppError> {
		if let Some(key) = event::read()?.as_key_press_event() {
			match key.code {
				KeyCode::Esc => {
					self.mode = AppMode::Idle;
			
					self.input = String::new();
					self.head = 0;
				},
				KeyCode::Char(c) => {
					self.input.insert(self.head, c);
					self.head += 1;
				},
				KeyCode::Enter => {
					if let Err(err) = self.handle_input() {
						self.set_err(err);
					} else {
						self.mode = AppMode::Idle;
					}

					self.input = String::new();
					self.head = 0;
				},
				KeyCode::Backspace => {
					if self.head > 0 {
						self.input.remove(self.head-1);
						self.head -= 1;
					}
				}
				KeyCode::Left => {
					if self.head > 0 {
						self.head -= 1
					}
				},
				KeyCode::Right => {
					if self.head < self.input.len() {
						self.head += 1;
					}
				},
				_ => {}
			}

		}

		Ok(())
	}

	fn handle_input(&mut self) -> Result<(), AppError> {
		match self.mode {
			AppMode::Input(t) => {
				match t {
					InputType::Damage => {
						self.status.damage(self.input.parse::<u8>()?);
						self.state_updated = true;
					},
					InputType::Heal => {
						self.status.heal(self.input.parse::<u8>()?);
						self.state_updated = true;
					},
					InputType::Cast => {
						let slot = self.input.parse::<usize>()?;

						if slot == 0 || slot > 9 {
							return Err(AppError::from(String::from("spell slots must be > 0 and < 9")))
						}

						if self.character.spell_slots[slot - 1] - self.status.used_slots[slot - 1] > 0 {
							self.status.cast(slot);
							self.state_updated = true;
						} else {
							return Err(AppError::from(String::from("not enough slots available")))
						}
					},
				}
			}
			_ => panic!("bug: handle_input shuold only be called while App.mode == AppMode::Input(AppType)")
		};

		Ok(())
	}

	fn handle_edit_select_events(&mut self) -> Result<(), AppError> {
		if let Some(key) = event::read()?.as_key_press_event() {
			match key.code {
				KeyCode::Up => self.edit_select_list_state.select_previous(),
				KeyCode::Down => self.edit_select_list_state.select_next(),
				KeyCode::Esc => {
					self.mode = AppMode::Idle;
					self.edit_select_list_state.select(Some(0));
				},
				KeyCode::Enter => self.mode = AppMode::Editing,
				_ => {},
			}
		}

		Ok(())
	}

	const BLOCK_COLOR: Color = SLATE.c800;

	const TAB_COLOR: Color = SLATE.c700;

	fn default_block() -> Block<'static> {
		Block::bordered()
			.bg(Self::BLOCK_COLOR)
			.title_alignment(Alignment::Center)
			.title_style(Modifier::BOLD)
	}

	fn render_popup(frame: &mut Frame, message: String) {
		let area = frame.area();
		let area = area.centered(Percentage(50),Percentage(10));

		frame.render_widget(Clear, area);

		frame.render_widget(
			Paragraph::new(message)
				.alignment(Alignment::Center)
				.block(Self::default_block().bg(RED.c900).title("Error").title_bottom("press any key to continue")),
			area,
		);
	}

	fn render_edit_list(&mut self, frame: &mut Frame) {
		let area = frame.area();
		let area = area.centered(Percentage(60), Length(5));

		let items = [
			"Character - semi-permanent traits that change rarely",
			"State - ephemeral values which may change on the fly",
			"Notes - notes about your character",
		];

		frame.render_widget(Clear, area);

		frame.render_stateful_widget(
			List::new(items)
				.block(
					Self::default_block()
					.title(" What do you want to edit? ")
					.title_bottom(" press ESC to quit ")
				)
				.highlight_style(Modifier::REVERSED)
				.highlight_symbol("> "),
			area,
			&mut self.edit_select_list_state);
	}

	fn render_header_name(&self, frame: &mut Frame, area: Rect) {
		let percent_lost = self.status.damage as f32 / self.character.max_hp as f32;
		let health_color = color_for_damage_percent_lost(percent_lost);

		frame.render_widget(
			Self::default_block().title(format!(" {} ", self.character.name)),
			area,
		);

		let [
			temp_health,
			health
		] = area.centered(Length(36), Length(6)).layout(
			&Layout::vertical([
				Length(1),
				Length(1),
			]).flex(layout::Flex::SpaceEvenly),
		);

		// todo: does not work well when temp_hp > 26 where the additional "+" exceeds the length constraint
		let temp_health_str = if self.status.temp_hp > 17 {
			format!(
				"{} +{}",
				(0..17).map(|_| "█").collect::<Vec<&str>>().join(" "),
				self.status.temp_hp - 17,
			)
		} else {
			(0..self.status.temp_hp).map(|_| "█").collect::<Vec<&str>>().join(" ")
		};

		frame.render_widget(
			Paragraph::new(temp_health_str.light_blue()),
			temp_health,
		);

		frame.render_widget(
			Gauge::default()
					.percent(100 - (percent_lost * 100.0) as u16)
				// .block(Block::new().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM))
				.gauge_style(health_color),
			health.centered(Length(36), Length(1)),
		);
	}

	fn render_header_info(&self, frame: &mut Frame, area: Rect) {
		let cols = (0..4).map(|i| if i < 3 { Fill(1) } else { Fill(2) });
		let rows = (0..2).map(|_| Fill(1));

		let horizontal = Layout::horizontal(cols).spacing(1);
		let vertical = Layout::vertical(rows).spacing(0);

		let rows = vertical.split(area);
		let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

		let data = [
			(" lvl ", self.character.level.to_string()),
			(" race ", self.character.race.clone()),
			(" class ", self.character.class.clone()),
			(" sub-class ", self.character.sub_class.clone()),
			(" luck ", self.status.luck.to_string()),
			(" AC ", self.character.armor_class.to_string()),
			(" speed ", self.character.speed.to_string()),
			(" languages ", self.character.languages.join(" ")),
		];

		for (i, cell) in cells.enumerate() {
			frame.render_widget(
				Paragraph::new(data[i].1.to_string())
					.alignment(Alignment::Center)
					.block(Self::default_block().title(data[i].0)),
			cell);
		}
	}

	fn render_header(&self, frame: &mut Frame, area: Rect) {
		let [
			name,
			info,
		] = area.layout(&Layout::horizontal([
			Length(40),
			Fill(1),
		]).spacing(1));

		self.render_header_name(frame, name);
		self.render_header_info(frame, info);
	}

	fn render_ability(&self, frame: &mut Frame, area: Rect, ability: &Ability) {
		let modifier: i16 = ((ability.score as i16) - 10) / 2;

		// the most amount of proficiencies for an ability if 5
		let mut lines = Vec::with_capacity(5);

		ability.name.proficiencies()
			.into_iter()
			.for_each(|p| {
				if ability.proficiencies.contains(&p) {
					lines.push(Line::from(Span::styled(p.clone(), Style::default().bg(Color::White).black())));
				} else {
					lines.push(Line::from(Span::styled(p.clone(), Style::default())));
				}
			}
		);

		let text = Text::from(lines);

		frame.render_widget(
			Paragraph::new(text)
				.alignment(Alignment::Center)
				.block(
					Self::default_block()
						.title(format!(" {} : {} ", ability.name, modifier).bold())
				),
				area);
	}

	fn render_abilities(&self, frame: &mut Frame, area: Rect) {
		let cols = (0..2).map(|_| Fill(1));
		let rows = (0..3).map(|_| Length(7));

		let horizontal = Layout::horizontal(cols).spacing(1);
		let vertical = Layout::vertical(rows).spacing(1);

		let rows = vertical.split(area);
		let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

		for (i, cell) in cells.enumerate() {
			self.render_ability(frame, cell, &self.character.abilities[i]);
		}
	}

	fn render_class_abilities(&self, frame: &mut Frame, area: Rect, has_rage: bool, has_magic: bool) {
		let [
			rages,
			spell_slots,
		] = area.layout(
			&Layout::vertical([
				Length(if has_rage { 3 } else { 0 }),
				Length(if has_magic { 16 } else { 0 }),
			]).spacing(1),
		);

		let rage_count = if self.status.used_rages > self.character.rages {
			0
		} else {
			self.character.rages - self.status.used_rages
		};

		frame.render_widget(
			Paragraph::new(rage_count.to_string())
				.alignment(Alignment::Center)
				.block(Self::default_block().title(" rages ")),
			rages);

		let cols = (0..3).map(|_| Fill(1));
		let rows = (0..3_).map(|_| Length(3));

		let horizontal = Layout::horizontal(cols).spacing(1);
		let vertical = Layout::vertical(rows).spacing(1);

		let rows = vertical.split(spell_slots);
		let slots = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

		for (i, slot) in slots.enumerate() {
			let available_slots = if self.character.spell_slots[i] >= self.status.used_slots[i] {
				self.character.spell_slots[i] - self.status.used_slots[i]
			} else {
				0
			};

			frame.render_widget(	
				Paragraph::new(available_slots.to_string())
					.alignment(Alignment::Center)
					.block(Self::default_block().title_alignment(Alignment::Left).title(format!(" {} ", utils::to_roman_numerals(i+1)))),
				slot,
			);
		}
	}

	fn render_coins(&self, frame: &mut Frame, area: Rect) {
		let coins: [Rect; 5] = area.layout(&Layout::horizontal((0..5).map(|_| Fill(1))));

		for (i, coin) in coins.iter().enumerate() {
			let title = match i {
				0 => " C ",
				1 => " S ",
				2 => " E ",
				3 => " G ",
				4 => " P ",
				_ => panic!("bug: length of coins shuold neevr exceed 5"),
			};

			frame.render_widget(
				Paragraph::new(self.status.coins[i].to_string())
					.block(Self::default_block()
					.title(title)),
				*coin);
		}
	}

	fn render_proficiencies(&self, frame: &mut Frame, area: Rect) {
		let lines: Vec<Line<'_>> = self.character.proficiencies.iter().map(|s| Line::from(s.clone())).collect();
		let text = Text::from(lines);
	
		frame.render_widget(Paragraph::new(text).block(Self::default_block().title(" proficiencies ")), area);
	}

	fn render_items(&mut self, frame: &mut Frame, area: Rect) {
		// todo: should support scroll bar
		let header = Row::new(["name", "count", "description"])
			.style(Style::default().bg(Self::BLOCK_COLOR))
			.bold();
		
		let mut items = Vec::<Row>::with_capacity(self.status.items.len());
		
		for item in &self.status.items {
			items.push(Row::new([item.name.to_string(), item.count.to_string(), item.description.to_string()]));
		}
		
		let table = Table::new(items, [Length(20), Length(10), Fill(1)])
			.block(Self::default_block())
			.header(header)
			.row_highlight_style(Style::new().on_white().bold())
			.column_spacing(1)
			.style(Color::White)
			.row_highlight_style(Style::default().bg(Self::TAB_COLOR));
		
		frame.render_stateful_widget(table, area, &mut self.item_table_state);
	}

	fn render_spells(&mut self, frame: &mut Frame, area: Rect) {
		// todo: should support scroll bar
		let header = Row::new(["name", "lvl", "description"])
			.style(Style::default().bg(Self::BLOCK_COLOR))
			.bold();
		
		let mut items = Vec::<Row>::with_capacity(self.character.spells.len());
		
		for spell in &self.character.spells {
			items.push(Row::new([
				spell.name.to_string(),
				if spell.level == 0 { String::from("C") } else { to_roman_numerals(spell.level as usize).to_string() },
				spell.description.clone()]));
		}
		
		let table = Table::new(items, [Length(20), Length(10), Fill(1)])
			.block(Self::default_block())
			.header(header)
			.row_highlight_style(Style::new().on_white().bold())
			.column_spacing(1)
			.style(Color::White)
			.row_highlight_style(Style::default().bg(Self::TAB_COLOR));
		
		frame.render_stateful_widget(table, area, &mut self.spell_table_state);
	}

	fn render_notes(&mut self, frame: &mut Frame, area: Rect) -> Result<(), AppError> {
		let data = std::fs::read_to_string(&self.notes_path)?;

		let paragraph = Paragraph::new(data)
			.wrap(Wrap{trim: false,})
			.scroll((self.note_scrollbar_state.get_position() as u16, 0))
			.block(Self::default_block());

		let n_lines = paragraph.line_count(area.width);

		frame.render_widget(
			paragraph,
			area);

		if n_lines > area.height as usize {
			self.note_scrollbar_state = self.note_scrollbar_state.content_length(n_lines - (area.height - 1) as usize);
			let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

			frame.render_stateful_widget(
				scrollbar,
				area.inner(Margin{
					vertical: 1,
					horizontal: 0,
				}),
				&mut self.note_scrollbar_state,
			);
		}

		Ok(())
	}

	fn render_tabs(&mut self, frame: &mut Frame, area: Rect) -> Result<(), AppError> {
		let items = if self.character.spells.is_empty() {
			vec!["items", "notes"]
		} else {
			vec!["items", "spells", "notes"]
		};

		let tab_index = match self.active_tab {
			Tab::Items => 0,
			Tab::Spells => 1,
			Tab::Notes => if self.character.spells.len() > 0 {
				2
			} else {
				1
			}
		};

		let tabs = Tabs::new(items)
			.highlight_style(Style::default().bg(Self::BLOCK_COLOR).fg(Color::White))
			.select(tab_index);

		frame.render_widget(tabs, area);

		let area = area
			.offset(Offset::new(0, 1))
			.resize(layout::Size {
				width: area.width,
				height: if area.height > 0 {
					area.height - 1
				} else {
					area.height
				},
			},
		);

		match self.active_tab {
			Tab::Items => self.render_items(frame, area),
			Tab::Spells => self.render_spells(frame, area),
			Tab::Notes => self.render_notes(frame, area)?,
		}

		Ok(())
	}

	const SHORT_BINDINGS_LIST: [(&'static str, &'static str); 7] = [
		("q", "quit"),
		("d", "take damage"),
		("h", "heal"),
		("r", "rage"),
		("c", "cast spell"),
		("e", "edit"),
		("?", "show help page"),
	];

	fn render_bindings(&self, frame: &mut Frame, area: Rect) {
		let constraints = (0..Self::SHORT_BINDINGS_LIST.len()).map(|_| Fill(1));
		let cells = Layout::horizontal(constraints).split(area).to_vec().into_iter();

		for (i, cell) in cells.enumerate() {
			let line = Line::from(vec![
				Span::styled(Self::SHORT_BINDINGS_LIST[i].0, Style::default()).black().bg(Color::White),
				" ".into(),
				Self::SHORT_BINDINGS_LIST[i].1.into(),
			]);	
			let text = Text::from(line);
			
			frame.render_widget(Paragraph::new(text), cell);
		}
	}

	fn render_input(&self, frame: &mut Frame, area: Rect, input_type: InputType) {
		let prompt = match input_type {
			InputType::Damage => "Damage",
			InputType::Heal => "Heal",
			InputType::Cast => "Cast Slot #",
		};

		let [
			prompt_rect,
			input,
		] = area.layout(&Layout::horizontal([
			Length(prompt.len() as u16),
			Fill(1),
		]).spacing(1));

		frame.render_widget(Paragraph::new(prompt.bg(Color::White).black()), prompt_rect);
		frame.render_widget(Paragraph::new(self.input.clone()), input);

		frame.set_cursor_position(Position::new(input.x + (self.head as u16), input.y));
	}

	fn render(&mut self, frame: &mut Frame) -> Result<(), AppError> {
		let area = frame.area();

		frame.render_widget(Clear, area);

		let [
			header,
			middle,
			bindings,
		] = area.layout(
			&Layout::vertical([
				Length(6),
				Fill(1),
				Length(1),
			],
		).spacing(1));

		let [
			left,
			right,
		] = middle.layout(
				&Layout::horizontal([
				Length(40),
				Fill(1),
			],
		).spacing(1));

		let has_rage = self.character.rages > 0;
		let has_magic = self.character.spell_slots.iter().sum::<u8>() > 0;
		let mut class_length = 0;

		if has_rage {
			class_length += 3;
		}

		if has_magic {
			class_length += 12;
		}

		let [
			left_top,
			left_upper_middle,
			left_lower_middle,
			left_bottom,
		] = left.layout(
			&Layout::vertical([
				Length(23),
				Length(class_length),
				Length(3),
				Fill(1)
			],
		).spacing(1).flex(layout::Flex::SpaceEvenly));

		self.render_header(frame, header);

		self.render_abilities(frame, left_top);
		self.render_class_abilities(frame, left_upper_middle, has_rage, has_magic);
		self.render_coins(frame, left_lower_middle);
		self.render_proficiencies(frame, left_bottom);

		self.render_tabs(frame, right)?;

		match self.mode {
			AppMode::Input(t) => self.render_input(frame, bindings, t),
			AppMode::EditSelection => {
				self.render_bindings(frame, bindings);
				self.render_edit_list(frame)
			},
			AppMode::Error => {
				self.render_bindings(frame, bindings);
				if let Some(err) = &self.err {
					Self::render_popup(frame, format!("{}", err));
					self.err = None;
				} else {
					Self::render_popup(frame, String::from("something wrong happened"));
				}
			},
			_ => self.render_bindings(frame, bindings),
		};

		Ok(())
	}

	fn render_help(&self, frame: &mut Frame) {
		let area = frame.area();

		let mut lines = vec![];

		lines.push("Basic keybindings");

		frame.render_widget(Clear, area);
		frame.render_widget(
			Paragraph::new("help text")
				.block(Self::default_block()
				.title("Help")),
			area);
	}
}
