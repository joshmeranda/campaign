use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Flex, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, TableState, Widget};
use ratatui::Frame;

use crate::error::AppError;

// todo: consider adding NoMatch vairnat to disitinguish between received a value and expecting more vs value awas received but not something that is handled
// todo: rename tosomething like HandleAction
pub enum HandleState {
    // The handler has received and delt wiuth the most recent event, and is expecting more.
    Handled,

    // The handler has recevied an event, but it does not have a use for it.
    Ignored,

    // The event handler is no longer receiving events, and the result is useable.
    Done,

    // The event handler is no longer receiving events, and the result ignored.
    Cancelled,
}

// todo: add default implementations for input handler for List and Table
pub trait InputHandler {
    // handle reads a keyboard events and returns Some(true) the InputHanlder should expect to continue taking input. In most cases, this will be when the handler reads ENTER.
    fn handle(&mut self) -> Result<HandleState, AppError> {
        if let Some(key) = event::read()?.as_key_press_event() {
            self.handle_event(key)
        } else {
            Ok(HandleState::Handled)
        }
    }

    fn handle_event(&mut self, event: KeyEvent) -> Result<HandleState, AppError>;
}

impl<H: InputHandler + ?Sized> InputHandler for &mut H {
    fn handle_event(&mut self, event: KeyEvent) -> Result<HandleState, AppError> {
        (**self).handle_event(event)
    }
}

pub struct InputHandlerChain<First, Second> {
    first: First,
    second: Second,
}

impl<First: InputHandler, Second: InputHandler> InputHandlerChain<First, Second> {
    pub fn new(first: First, second: Second) -> Self {
        Self { first, second }
    }

    pub fn chain<Next: InputHandler>(self, next: Next) -> InputHandlerChain<Self, Next> {
        InputHandlerChain::new(self, next)
    }
}

impl<First: InputHandler, Second: InputHandler> InputHandler
    for InputHandlerChain<First, Second>
{
    fn handle_event(&mut self, event: KeyEvent) -> Result<HandleState, AppError> {
        match self.first.handle_event(event)? {
            HandleState::Ignored => self.second.handle_event(event),
            state => Ok(state),
        }
    }
}

// todo: need to support text being longer than the input widget width
#[derive(Default)]
pub struct InputState {
    buffer: String,
    head: u16,

    // Position should point to the start of the input buffer on the terminal. The cursor position can be determined by adding InputState.head to position.x. If position is None the Input does not have focus and should not move the cursor.
    position: Option<Position>,
}

impl InputState {
    pub fn move_cursor(&mut self, n: i8) {
        self.head = self
            .head
            .saturating_add_signed(n as i16)
            .min(self.buffer.len() as u16);
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.head = 0;
        self.position = None;
    }

    pub fn value(&self) -> &str {
        &self.buffer
    }

    pub fn set_position(&mut self, position: Option<Position>) {
        self.position = position;
    }

    pub fn set_cursor_position(&self, frame: &mut Frame) {
        if let Some(p) = self.position {
            frame.set_cursor_position(Position {
                x: (p.x + self.head),
                ..p
            });
        }
    }
}

impl InputHandler for InputState {
    fn handle_event(&mut self, event: KeyEvent) -> Result<HandleState, AppError> {
        match event.code {
            KeyCode::Char(c) => {
                if c == 'u' && event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.buffer.truncate(0); // todo: this should only remove the characters before self.head not evrything
                    self.head = 0;
                } else {
                    self.buffer.insert(self.head as usize, c);
                    self.move_cursor(1);
                }
            }

            KeyCode::Right => self.move_cursor(1),
            KeyCode::Left => self.move_cursor(-1),

            KeyCode::Backspace => {
                if self.head != 0 {
                    self.buffer.remove(self.head.saturating_sub(1) as usize);
                }

                self.move_cursor(-1);
            }

            KeyCode::Enter => return Ok(HandleState::Done),

            KeyCode::Esc => return Ok(HandleState::Cancelled),

            _ => return Ok(HandleState::Ignored),
        }

        Ok(HandleState::Handled)
    }
}

pub struct Input {}

impl Input {
    pub fn new() -> Self {
        Input {}
    }
}

impl StatefulWidget for Input {
    type State = InputState;

    // After calling render, the caller should also call set_cursor_position to ensure that the cursor is visible and in the correct position.
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
        state.set_position(Some(rect.as_position()));
        let line = Line::from(state.value());

        line.render(rect, buffer);
    }
}

pub struct MultiInputState<const N: usize> {
    active_input: usize,
    inputs: [InputState; N],
}

impl<const N: usize> MultiInputState<N> {
    fn next_input(&mut self) {
        if self.active_input == N - 1 {
            self.active_input = 0;
        } else {
            self.active_input += 1;
        }
    }

    fn previous_input(&mut self) {
        if self.active_input == 0 {
            self.active_input = N - 1;
        } else {
            self.active_input -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.active_input = 0;

        for i in 0..N {
            self.inputs[i].reset()
        }
    }

    pub fn set_cursor_position(&self, frame: &mut Frame) {
        self.inputs[self.active_input].set_cursor_position(frame);
    }

    pub fn values(&self) -> [&str; N] {
        let mut values: [&str; N] = [""; N];

        for i in 0..N {
            values[i] = self.inputs[i].value();
        }

        values
    }
}

impl<const N: usize> Default for MultiInputState<N> {
    fn default() -> Self {
        MultiInputState {
            active_input: 0,
            inputs: core::array::from_fn(|_| InputState::default()),
        }
    }
}

impl<const N: usize> InputHandler for MultiInputState<N> {
    fn handle_event(&mut self, event: KeyEvent) -> Result<HandleState, AppError> {
        match event.code {
            KeyCode::Tab => self.next_input(),
            KeyCode::BackTab => self.previous_input(),

            KeyCode::Enter => {
                if self.active_input == N - 1 {
                    return Ok(HandleState::Done);
                } else {
                    self.next_input();
                }
            }

            KeyCode::Esc => {
                self.reset();
                return Ok(HandleState::Cancelled);
            }

            _ => return self.inputs[self.active_input].handle_event(event),
        };

        Ok(HandleState::Handled)
    }
}

pub struct MultiInput<'a, const N: usize> {
    prompts: [&'a str; N],
    block: Option<Block<'a>>,
}

impl<'a, const N: usize> MultiInput<'a, N> {
    pub fn new(prompts: [&'a str; N]) -> Self {
        MultiInput {
            prompts: prompts,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn render_input(
        &self,
        area: Rect,
        buffer: &mut Buffer,
        state: &mut MultiInputState<N>,
        i: usize,
    ) {
        let prompt = self.prompts[i];
        let line = Line::from(if i == state.active_input {
            prompt.bg(Color::White).fg(Color::Black)
        } else {
            Span::from(prompt)
        });

        let [left, right] =
            area.layout(&Layout::horizontal([Length(prompt.len() as u16), Fill(1)]).spacing(1));

        if i == state.active_input {
            state.inputs[state.active_input].set_position(Some(right.as_position()))
        }

        line.render(left, buffer);
        Input::new().render(right, buffer, &mut state.inputs[i]);
    }
}

impl<const N: usize> StatefulWidget for MultiInput<'_, N> {
    type State = MultiInputState<N>;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let mut area = area;

        if let Some(block) = &self.block {
            block.render(area, buffer);
            area = area.inner(Margin {
                horizontal: 1,
                vertical: 1,
            })
        }

        let areas: [Rect; N] =
            area.layout(&Layout::vertical((0..N).map(|_| Length(1))).flex(Flex::SpaceEvenly));

        for (i, input_area) in areas.iter().enumerate() {
            self.render_input(*input_area, buffer, state, i);
        }
    }
}

impl InputHandler for TableState {
    fn handle_event(&mut self, event: KeyEvent) -> Result<HandleState, AppError> {
        match event.code {
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),

            KeyCode::Enter => return Ok(HandleState::Done),
            KeyCode::Esc => return Ok(HandleState::Cancelled),

            _ => return Ok(HandleState::Ignored),
        }

        Ok(HandleState::Handled)
    }
}

#[cfg(test)]
mod test_input_state {
    use super::*;

    #[test]
    fn test_mode_cursor() {
        struct Test<'a> {
            title: &'a str,

            head: u16,
            delta: i8,

            expected: u16,
        }

        let cases = vec![
            Test {
                title: "right 1 with Some(0)",
                head: 0,
                delta: 1,
                expected: 1,
            },
            Test {
                title: "right 1 with Some(u16::MAX)",
                head: u16::MAX,
                delta: 1,
                expected: u16::MAX,
            },
            Test {
                title: "left 1 with Some(0)",
                head: 0,
                delta: -1,
                expected: 0,
            },
            Test {
                title: "left 1 with Some(u16::MAX)",
                head: u16::MAX,
                delta: -1,
                expected: u16::MAX - 1,
            },
        ];

        for c in cases {
            let mut state = InputState {
                head: c.head,
                ..InputState::default()
            };

            state.move_cursor(c.delta);

            assert_eq!(c.expected, state.head, "{}", c.title);
        }
    }
}
