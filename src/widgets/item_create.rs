use ratatui::buffer::Buffer;
use ratatui::layout::{Flex, Rect, Layout, Margin};
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Widget, StatefulWidget, Block};
use ratatui::Frame;
use ratatui::text::{Line, Span};
use ratatui::layout::Constraint::{Fill, Length};
use crossterm::event::{KeyCode, KeyEvent};

use crate::error::AppError;
use crate::types::Item;
use crate::widgets::input::{Input, InputState, InputHandler};

#[derive(Copy, Clone, PartialEq)]
enum CreateField {
	Name,
	Count,
	Description
}

impl CreateField {
	fn next(&self) -> CreateField {
		match self {
			Self::Name => Self::Count,
			Self::Count => Self::Description,
			Self::Description => Self::Name,
		}
	}

	fn previous(&self) -> CreateField {
		match self {
			Self::Name => Self::Description,
			Self::Count => Self::Name,
			Self::Description => Self::Count,
		}
	}
}

pub struct CreateItemState {
	focus: CreateField,

	name_input_state: InputState,
	count_input_state: InputState,
	description_input_state: InputState
}

impl CreateItemState {
	pub fn reset(&mut self) {
		self.focus = CreateField::Name;

		self.name_input_state.reset();
		self.count_input_state.reset();
		self.description_input_state.reset();
	}

	pub fn set_cursor_position(&self, frame: &mut Frame) {
		match self.focus {
			CreateField::Name => self.name_input_state.set_cursor_position(frame),
			CreateField::Count => self.count_input_state.set_cursor_position(frame),
			CreateField::Description => self.description_input_state.set_cursor_position(frame),
		}
	}

	pub fn item(&self) -> Result<Option<Item>, AppError> {
		if self.name_input_state.value().is_empty() {
			return Err(AppError::Error(String::from("item name must not be empty")))
		}

		let count = self.count_input_state.value().parse::<u8>()?;

		Ok(Some(Item{
			name: String::from(self.name_input_state.value()),
			count: count,
			description: String::from(self.name_input_state.value()),
		}))
	}
}

impl Default for CreateItemState {
	fn default() -> Self {
		CreateItemState {
			focus: CreateField::Name,
			name_input_state: InputState::default(),
			count_input_state: InputState::default(),
			description_input_state: InputState::default(),
		}
	}
}

impl InputHandler for CreateItemState {
	fn handle_event(&mut self, event: KeyEvent) -> Result<bool, AppError> {
		match event.code {
			KeyCode::Tab => self.focus = self.focus.next(),
			KeyCode::BackTab => self.focus = self.focus.previous(),

			KeyCode::Enter => if ! matches!(self.focus, CreateField::Description) {
				self.focus = self.focus.next();
			} else {
				return Ok(false);
			}

			KeyCode::Esc => {
				self.reset();
				return Ok(false)
			},

			_ => return match self.focus {
				CreateField::Name => self.name_input_state.handle_event(event),
				CreateField::Count => self.count_input_state.handle_event(event),
				CreateField::Description => self.description_input_state.handle_event(event),
			},
		}

		Ok(true)
	}
}

#[derive(Default)]
pub struct CreateItem<'a> {
	block: Option<Block<'a>>
}

impl <'a> CreateItem<'a> {
	pub fn block(mut self, block: Block<'a>) -> Self {
		self.block = Some(block);
		self
	}

	fn render_input(&self, area: Rect, buffer: &mut Buffer, state: &mut CreateItemState, field: CreateField) {
		let (prompt, input_state)  = match field {
			CreateField::Name => ("name", &mut state.name_input_state),
			CreateField::Count => ("count", &mut state.count_input_state),
			CreateField::Description => ("description", &mut state.description_input_state),
		};

		let line = Line::from(if field == state.focus {
			prompt.bg(Color::White).fg(Color::Black)
		} else {
			Span::from(prompt)
		});

		let [
			left, right
		] = area.layout(&Layout::horizontal([
			Length(prompt.len() as u16),
			Fill(1),
		]).spacing(1));

		line.render(left, buffer);
		Input::new().render(right, buffer, input_state);
	}
}

impl StatefulWidget for CreateItem<'_> {
	type State = CreateItemState;

	fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
		let mut area = area;

		if let Some(block) = &self.block {
			block.render(area, buffer);
            area = area.inner(Margin{horizontal: 1, vertical: 1})
		}
		
		let [
			top, middle, bottom
		] = area.layout(&Layout::vertical([
			Length(1),
			Length(1),
			Length(1)
		]).flex(Flex::SpaceEvenly));

		self.render_input(top, buffer, state, CreateField::Name);
		self.render_input(middle, buffer, state, CreateField::Count);
		self.render_input(bottom, buffer, state, CreateField::Description);
	}
}