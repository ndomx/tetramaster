use crate::ui::ascii::{
    ascii_view::AsciiView, constants::{CARD_HEIGHT, CARD_INNER_WIDTH, CARD_WIDTH},
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
        let top = "─".repeat(CARD_INNER_WIDTH);
        let middle = " ".repeat(CARD_INNER_WIDTH);
        let bottom = "─".repeat(CARD_INNER_WIDTH);
        match line {
            0 => format!("┌{top}┐"),
            1..=7 => format!("│{middle}│"),
            _ => format!("└{bottom}┘"),
        }
    }
}
