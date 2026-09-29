use crossterm::event::{KeyEvent, KeyCode};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Text, Line};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::style::Stylize;

use crate::widgets::input::{InputHandler, HandleState};
use crate::error::AppError;

#[derive(Default)]
pub struct OptionsState {
	selected: Option<char>,
}

pub type OptionBinding<'a> = (char, &'a str);

impl OptionsState {
	pub fn selected(&self) -> Option<char> {
		self.selected
	}

	pub fn reset(&mut self) {
		self.selected = None;
	}
}

impl InputHandler for OptionsState {
	fn handle_event(&mut self, event: KeyEvent) -> Result<HandleState, AppError> {
		match event.code {
			KeyCode::Esc => return Ok(HandleState::Cancelled),
			KeyCode::Char(c) =>{
				self.selected = Some(c);
				return Ok(HandleState::Done);
			},

			_ => {},
		}

		Ok(HandleState::Expecting)
	}
}

pub struct Options<'a, const N: usize> {
	options: [(char, &'a str); N],
	block: Option<Block<'a>>,
}

impl <'a, const N: usize> Options<'a, N> {
	pub fn new(options: [OptionBinding<'a>; N]) -> Self {
		Options {
			options: options,
			block: None,
		}
	}

	pub fn block(mut self, block: Block<'a>) -> Self {
		self.block = Some(block);
		self
	}
}

impl <'a, const N: usize> Widget for Options<'a, N> {
	fn render(self, area: Rect, buffer: &mut Buffer) {
		let lines: Vec<Line> = self.options.iter()
			.map(|i| Line::from(format!("({}) {}", i.0.bold(), i.1)))
			.collect();
		let text = Text::from(lines);

		let mut paragraph = Paragraph::new(text);

		if let Some(block) = self.block {
			paragraph = paragraph.block(block);
		}

		paragraph.render(area, buffer);
	}
}