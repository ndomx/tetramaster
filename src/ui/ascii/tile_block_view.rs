use crossterm::style::{Color, Stylize};

use crate::{
    ui::ascii::{
        ascii_view::AsciiView,
        constants::{CARD_HEIGHT, CARD_WIDTH},
    },
};

pub struct TileBlockView {}

impl AsciiView for TileBlockView {
    fn width(&self) -> usize {
        CARD_WIDTH
    }

    fn height(&self) -> usize {
        CARD_HEIGHT
    }

    fn line(&self, line: usize) -> String {
        match line {
            0 => "┌─────────────┐".on(Color::Grey).to_string(),
            1 => "│             │".on(Color::Grey).to_string(),
            2 => "│             │".on(Color::Grey).to_string(),
            3 => "│             │".on(Color::Grey).to_string(),
            4 => "│             │".on(Color::Grey).to_string(),
            5 => "│             │".on(Color::Grey).to_string(),
            6 => "│             │".on(Color::Grey).to_string(),
            7 => "│             │".on(Color::Grey).to_string(),
            _ => "└─────────────┘".on(Color::Grey).to_string(),
        }
    }
}
