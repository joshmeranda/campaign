#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
struct WindowedString {
    buffer: String,

    head: usize,
    window: Window,
}

impl WindowedString {
	fn next(&mut self) {
		self.head = self.head.saturating_add(1).min(self.buffer.len());

		if !self.window.contains(self.head) {
			self.window = self.window.shift(1);
		}
	}

	fn previous(&mut self) {
		self.head = self.head.saturating_sub(1);

		if !self.window.contains(self.head) {
			self.window = self.window.shift(-1)
		}
	}

	fn value(&self) -> String {
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
}

#[cfg(test)]
mod teest_window {
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
		assert_eq!(ws.value(), String::from("…234…"))
	}
}