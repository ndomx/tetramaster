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
            0 => "┌─────────────┐".to_string(),
            1 => "│x x x x x x x│".to_string(),
            2 => "│ x x x x x x │".to_string(),
            3 => "│x x x x x x x│".to_string(),
            4 => "│ x x x x x x │".to_string(),
            5 => "│x x x x x x x│".to_string(),
            6 => "│ x x x x x x │".to_string(),
            7 => "│x x x x x x x│".to_string(),
            _ => "└─────────────┘".to_string(),
        }
    }
}
