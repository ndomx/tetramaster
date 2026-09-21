use crossterm::style::{Color, Stylize};

use crate::{
    models::{card::Card, direction::Direction},
    ui::ascii::{
        ascii_view::AsciiView,
        constants::{
            CARD_CONTENT_SPACE, CARD_HEIGHT, CARD_INNER_WIDTH, CARD_WIDTH, MAX_CARD_WRITABLE_IDX,
        },
    },
};

const FRONT_COLOR: Color = Color::Yellow;

pub struct TileCardView<'a> {
    card: &'a Card,
    back_color: Color,
}

impl<'a> TileCardView<'a> {
    pub fn new(card: &'a Card, back_color: Color) -> Self {
        Self { card, back_color }
    }

    fn top_line(&self) -> String {
        let mut northwest = " ";
        let mut north = " ";
        let mut northeast = " ";

        self.card.facing().iter().for_each(|d| match *d {
            Direction::NW => northwest = "◤",
            Direction::N => north = "▲",
            Direction::NE => northeast = "◥",
            _ => {}
        });

        format!("│{}{:^CARD_CONTENT_SPACE$}{}│", northwest, north, northeast)
    }

    fn mid_line(&self) -> String {
        let mut west = " ";
        let mut east = " ";

        self.card.facing().iter().for_each(|d| match *d {
            Direction::W => west = "◀",
            Direction::E => east = "▶",
            _ => {}
        });

        format!(
            "│{}{:^CARD_CONTENT_SPACE$}{}│",
            west,
            self.card.format_stats(),
            east
        )
    }

    fn bottom_line(&self) -> String {
        let mut southwest = " ";
        let mut south = " ";
        let mut southeast = " ";

        self.card.facing().iter().for_each(|d| match *d {
            Direction::SW => southwest = "◣",
            Direction::S => south = "▼",
            Direction::SE => southeast = "◢",
            _ => {}
        });

        format!("│{southwest}{south:^CARD_CONTENT_SPACE$}{southeast}│")
    }

    fn name_line(&self) -> String {
        format!("│ {:^CARD_CONTENT_SPACE$} │", self.card.asset.name)
    }
}

impl<'a> AsciiView for TileCardView<'a> {
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
            1 => self.top_line(),
            2..=5 => format!("│{middle}│"),
            6 => self.name_line(),
            7 => self.mid_line(),
            8..MAX_CARD_WRITABLE_IDX => format!("│{middle}│"),
            MAX_CARD_WRITABLE_IDX => self.bottom_line(),
            _ => format!("└{bottom}┘"),
        }
        .on(self.back_color)
        .with(FRONT_COLOR)
        .to_string()
    }
}
