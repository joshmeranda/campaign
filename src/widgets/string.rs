#[derive(Default, Debug, PartialEq, Copy, Clone)]
struct Window {
    start: usize,
    width: usize,
}

impl Window {
	fn shift(&self, n: isize) -> Self {
		Window {
			start: self.start.saturating_add_signed(n),
			..*self
		}
	}

	fn contains(&self, n: usize) -> bool {
		n >= self.start && n < self.start + self.width
	}
}

#[derive(Default, Debug, PartialEq)]
pub struct WindowedString {
    buffer: String,

    head: usize,

	window: Window,
}

impl WindowedString {
	pub fn next(&mut self) {
		self.head = self.head.saturating_add(1).min(self.buffer.len());

		if !self.window.contains(self.head) {
			self.window = self.window.shift(1);
		}
	}

	pub fn previous(&mut self) {
		self.head = self.head.saturating_sub(1);

		if !self.window.contains(self.head) {
			self.window = self.window.shift(-1)
		}
	}

	pub fn insert(&mut self, c: char) {
		self.buffer.insert(self.head, c);
		self.next();
	}

	pub fn remove(&mut self) {
		self.buffer.remove(self.head);
		self.previous();
	}

	pub fn remove_to_start(&mut self) {
		self.buffer = String::from(&self.buffer[self.head..]);
		self.head = 0;
	}

	pub fn value(&self) -> String { // todo: would love to return an &str here
		if self.buffer.len() < self.window.width {
			return self.buffer.clone();
		}

		let mut s = String::from(&self.buffer[self.window.start..self.window.start + self.window.width]);

		if self.window.start > 0 {
			s.replace_range(0..1, "…");
		}

		if self.window.start + self.window.width < self.buffer.len() {
			s.replace_range(s.len()-1..s.len(), "…");
		}

		s
    }

	pub fn full_value(&self) -> &str {
		&self.buffer
	}

	pub fn set_width(&mut self, width: usize) {
		let start = if width >= self.buffer.len().saturating_sub(1) {
			self.window.start
		} else if self.window.start + width > self.buffer.len() {
			self.buffer.len() - width
		} else if self.window.start + width <= self.head {
			if self.head == self.buffer.len() {
				self.head - width
			} else {
				self.head - width + 1
			}
		} else {
			self.window.start
		};

		self.window = Window {
			start,
			width
		};
	}

	pub fn cursor_offset(&self) -> u16 {
		(self.head - self.window.start) as u16
	}
}

#[cfg(test)]
mod test_window {
	use super::Window;

	#[test]
	fn test_contains() {
		struct Test<'a> {
			title: &'a str,

			window: Window,
			target: usize,

			expected: bool
		}

		let cases = vec![
			Test{
				title: "contains beggining",
				window: Window{
					start: 0,
					width: 5,
				},
				target: 0,
				expected: true,
			},
			Test{
				title: "contains middle",
				window: Window{
					start: 0,
					width: 5,
				},
				target: 3,
				expected: true,
			},
			Test{
				title: "contains end",
				window: Window{
					start: 0,
					width: 5,
				},
				target: 4,
				expected: true,
			},

			Test{
				title: "does not contain after end",
				window: Window{
					start: 0,
					width: 5,
				},
				target: 6,
				expected: false,
			},
			Test{
				title: "does not contain before end",
				window: Window{
					start: 5,
					width: 10,
				},
				target: 4,
				expected: false,
			},

		];

		for c in cases {
			assert_eq!(c.expected, c.window.contains(c.target), "{}", c.title);
		}
	}

	#[test]
	fn test_shift() {
		let mut window = Window{
			start: 0,
			width: 5,
		};

		window = window.shift(-1);

		assert_eq!(
			window,
			Window{
				start: 0,
				width: 5,
			},
		);

		window = window.shift(1);
		assert_eq!(
			window,
			Window{
				start: 1,
				width: 5,
			},
		);

		window = window.shift(3);
		assert_eq!(
			window,
			Window{
				start: 4,
				width: 5,
			},
		);

		window = window.shift(-1);
		assert_eq!(
			window,
			Window{
				start: 3,
				width: 5,
			},
		);
	}
}

#[cfg(test)]
mod test_windowed_string {
    use crate::widgets::string::{Window, WindowedString};

	#[test]
	fn test_window_next_previous() {
		let mut ws = WindowedString{
			buffer: String::from("0123456789"),

			head: 0,
			window: Window {
				start: 0,
				width: 5,
			}
		};
		assert_eq!(ws.value(), String::from("0123…"));

		ws.next();

		assert_eq!(
			ws,
			WindowedString{
				buffer: String::from("0123456789"),

				head: 1,
				window: Window{
					start: 0,
					width: 5,
				},
			},
		);
		assert_eq!(ws.value(), String::from("0123…"));

		ws.next();
		ws.next();
		ws.next();
		ws.next();

		assert_eq!(
			ws,
			WindowedString{
				buffer: String::from("0123456789"),

				head: 5,
				window: Window{
					start: 1,
					width: 5,
				},
			},
		);
		assert_eq!(ws.value(), String::from("…234…"));

		ws.previous();

		assert_eq!(
			ws,
			WindowedString{
				buffer: String::from("0123456789"),

				head: 4,
				window: Window{
					start: 1,
					width: 5,
				},
			},
		);
		assert_eq!(ws.value(), String::from("…234…"));

		ws.next();
		ws.next();
		ws.next();
		ws.next();
		ws.next();

		assert_eq!(
			ws,
			WindowedString{
				buffer: String::from("0123456789"),

				head: 9,
				window: Window{
					start: 5,
					width: 5,
				},
			},
		);
		assert_eq!(ws.value(), String::from("…6789"));
	}

	#[test]
	fn test_set_width() {
		struct Test<'a> {
			title: &'a str,

			string: WindowedString,
			width: usize,

			expected: Window,
			value: String,
		}

		let cases = vec![
			Test{
				title: "when resize empty string at start",
				string: WindowedString{
					buffer: String::new(),
					head: 0,
					window: Window::default(),
				},
				width: 10,
				expected: Window{
					start: 0,
					width: 10,
				},
				value: String::new(),
			},

			Test{
				title: "when string has None window at start and fits",
				string: WindowedString{
					buffer: String::from("0123456789"),
					head: 0,
					window: Window::default(),
				},
				width: 15,
				expected: Window{
					start: 0,
					width: 15,
				},
				value: String::from("0123456789")
			},
			Test{
				title: "when string has None window at start and does not fit",
				string: WindowedString{
					buffer: String::from("0123456789"),
					head: 0,
					window: Window::default(),
				},
				width: 5,
				expected: Window{
					start: 0,
					width: 5,
				},
				value: String::from("0123…")
			},

			Test{
				title: "when shrink in middle",
				string: WindowedString{
					buffer: String::from("0123456789"),
					head: 4,
					window: Window{
						start: 2,
						width: 5,
					},
				},
				width: 4,
				expected: Window{
					start: 1,
					width: 4,
				},
				value:String::from("…23…"),
			},
			Test{
				title: "when shrink at end",
				string: WindowedString{
					buffer: String::from("0123456789"),
					head: 9,
					window: Window{
						start: 2,
						width: 8,
					},
				},
				width: 5,
				expected: Window{
					start: 5,
					width: 5,
				},
				value: String::from("…6789"),
			},

			Test{
				title: "when grow to include end and before",
				string: WindowedString {
					buffer: String::from("0123456789"),
					head: 7,
					window: Window{
						start: 5,
						width: 5,
					},
				},
				width: 8,
				expected: Window{
					start: 2,
					width: 8,
				},
				value: String::from("…3456789"),
			}
		];

		for mut c in cases {
			if c.title != "when shrink at end" {
				continue
			}

			c.string.set_width(c.width);

			assert_eq!(c.string.window, c.expected, "{}", c.title);
			assert_eq!(c.string.value(), c.value, "{}", c.title);
		}
	}
}