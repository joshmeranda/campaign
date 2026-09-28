use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Widget, StatefulWidget};
use ratatui::buffer::Buffer;

use crate::error::AppError;

pub trait InputHandler {
	// handle returns Some(true) the InputHanlder should expect to continue taking input. In most cases, this will be when the handler reads ENTER.
	fn handle(&mut self) -> Result<bool, AppError>;
}

#[derive(Default)]
pub struct InputState {
	buffer: String,
	head: u16,

	// Offset should point to the start of the input buffer. The buffer position can be determined by adding InputState.head to the ofset x.
	position: Option<Position>,
}

impl InputState {
	pub fn move_cursor(&mut self, n: i8) {
		// print!("{} {} {} {}|", self.buffer, self.head, n, self.head.saturating_add_signed(n as i16).min(self.buffer.len() as u16));
		self.head = self.head.saturating_add_signed(n as i16).min(self.buffer.len() as u16);
	}

	pub fn reset(&mut self) {
		self.buffer.clear();
		self.head = 0;
		self.position = None;
	}

	pub fn value(&self) -> &str {
		&self.buffer
	}

	pub fn set_cursor_position(&self, frame: &mut Frame) {
		if let Some(p) = self.position {
			frame.set_cursor_position(Position{
				x: (p.x + self.head),
				..p
			});
		}
	}
}

impl InputHandler for InputState {
	fn handle(&mut self) -> Result<bool, AppError> {
		if let Some(key) = event::read()?.as_key_press_event() {
			match key.code {
				KeyCode::Char(c) => {
					self.buffer.insert(self.head as usize, c);
					self.move_cursor(1);
				},

				KeyCode::Right => self.move_cursor(1),
				KeyCode::Left => self.move_cursor(-1),

				KeyCode::Backspace => {
					if self.head != 0 {
						self.buffer.remove(self.head.saturating_sub(1) as usize);
					}
	
					self.move_cursor(-1);
				},

				KeyCode::Enter => return Ok(false),

				_ => {},
			}
		}

		Ok(true)
	}
}

pub struct Input {
}

impl Input {
	pub fn new() -> Self {
		Input{}
	}
}

impl StatefulWidget for Input {
	type State = InputState;

	// After calling render, thje caller should also call set_cursor_position to ensure that the cursor is visible and in the correct position.
	//
	// ```
	// use ratatui::Frame;
	//
	// let mut frame  = &Frame{...}
	// let rect = Rect{...}
	// let mut state = &InputState::default();
	// let input = Input::new();
	// 
	// frame.render_stateful_widget(input, rect);
	// state.set_cursor(frame);
	// ```
	fn render(self, rect: Rect, buffer: &mut Buffer, state: &mut Self::State) {
		state.position = Some(rect.as_position());
		let paragraph = Line::from(state.value());

		paragraph.render(rect, buffer)
	}
}

#[cfg(test)]
mod test_input_state {
	use super::*;

	#[test]
	fn test_mode_cursor() {
		struct Test<'a>  {
			title: &'a str,

			head: u16,
			delta: i8,

			expected: u16,
		}

		let cases = vec![
			Test{
				title: "right 1 with Some(0)",
				head: 0,
				delta: 1,
				expected: 1,
			},
			Test{
				title: "right 1 with Some(u16::MAX)",
				head: u16::MAX,
				delta: 1,
				expected: u16::MAX,
			},
			Test{
				title: "left 1 with Some(0)",
				head: 0,
				delta: -1,
				expected: 0,
			},
			Test{
				title: "left 1 with Some(u16::MAX)",
				head: u16::MAX,
				delta: -1,
				expected: u16::MAX - 1,
			},
		];

		for c in cases {
			let mut state = InputState{
				head: c.head,
				..InputState::default()
			};

			state.move_cursor(c.delta);

			assert_eq!(c.expected, state.head, "{}", c.title);
		}
	}
}