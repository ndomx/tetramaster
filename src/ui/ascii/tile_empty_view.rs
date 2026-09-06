use crate::{
    ui::ascii::{
        ascii_view::AsciiView,
        constants::{CARD_HEIGHT, CARD_WIDTH},
    },
};

pub struct TileEmptyView {}

impl AsciiView for TileEmptyView {
    fn width(&self) -> usize {
        CARD_WIDTH
    }

    fn height(&self) -> usize {
        CARD_HEIGHT
    }

    fn line(&self, line: usize) -> String {
        match line {
            0 => "┌─────────────┐".to_string(),
            1 => "│             │".to_string(),
            2 => "│             │".to_string(),
            3 => "│             │".to_string(),
            4 => "│             │".to_string(),
            5 => "│             │".to_string(),
            6 => "│             │".to_string(),
            7 => "│             │".to_string(),
            _ => "└─────────────┘".to_string(),
        }
    }
}
