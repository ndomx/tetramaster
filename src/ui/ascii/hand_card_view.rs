use crossterm::style::Stylize;

use crate::{
    models::{card::Card, direction::Direction},
    ui::ascii::{
        ascii_view::AsciiView,
        constants::{CARD_HEIGHT, CARD_WIDTH},
    },
};

pub struct HandCardView<'a> {
    card: &'a Card,
}

impl<'a> HandCardView<'a> {
    pub fn new(card: &'a Card) -> Self {
        Self { card }
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

        let width = self.width();
        format!("│{northwest}{north:^width$}{northeast}│")
    }

    fn mid_line(&self) -> String {
        let mut west = " ";
        let mut east = " ";

        self.card.facing().iter().for_each(|d| match *d {
            Direction::W => west = "◀",
            Direction::E => east = "▶",
            _ => {}
        });

        let width = self.width();
        format!("│{}{:^width$}{}│", west, self.card.format_stats(), east)
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

        let width = self.width();
        format!("│{southwest}{south:^width$}{southeast}│")
    }
}

impl<'a> AsciiView for HandCardView<'a> {
    fn width(&self) -> usize {
        CARD_WIDTH
    }

    fn height(&self) -> usize {
        CARD_HEIGHT
    }

    fn line(&self, line: usize) -> String {
        match line {
            0 => "┌─────────────┐".to_string(),
            1 => self.top_line(),
            2 => "│             │".to_string(),
            3 => format!("│ {:^CARD_WIDTH$} │", self.card.asset.name),
            4 => self.mid_line(),
            5 => "│             │".to_string(),
            6 => "│             │".to_string(),
            7 => self.bottom_line(),
            _ => "└─────────────┘".to_string(),
        }
        .on_blue()
        .to_string()
    }
}
