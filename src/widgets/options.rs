use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::layout::Constraint::{Fill, Length, Percentage};
use ratatui::style::Stylize;
use ratatui::layout::Margin;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::error::AppError;
use crate::widgets::input::{HandleState, InputHandler};

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
            KeyCode::Esc => Ok(HandleState::Cancelled),
            KeyCode::Char(c) => {
                self.selected = Some(c);
                Ok(HandleState::Done)
            }

            _ => Ok(HandleState::Ignored),
        }
    }
}

pub struct Options<'a, const N: usize> {
    options: [(char, &'a str); N],
    block: Option<Block<'a>>,
}

impl<'a, const N: usize> Options<'a, N> {
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

impl<'a, const N: usize> Widget for Options<'a, N> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let lines: Vec<Line> = self
            .options
            .iter()
            .map(|i| Line::from(format!("({}) {}", i.0.bold(), i.1)))
            .collect();

        let text = Text::from(lines);
        let mut area = area;

        if let Some(block) = self.block {
            block.render(area, buffer);
            area = area.inner(Margin { horizontal: 1, vertical: 1 })
        }

        Paragraph::new(text).alignment(Alignment::Center)
            .render(area.centered(Fill(1), Length(self.options.len() as u16)), buffer);
    }
}
