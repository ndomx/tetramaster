use crossterm::style::{Color, Stylize};

use crate::{
    models::{core::geometry::Direction, session::CardSnapshot},
    ui::ascii::{
        ascii_view::AsciiView,
        constants::{
            CARD_CONTENT_SPACE, CARD_HEIGHT, CARD_INNER_WIDTH, CARD_WIDTH, MAX_CARD_WRITABLE_IDX,
        },
    },
};

const FRONT_COLOR: Color = Color::Yellow;

pub struct CardTileView<'a> {
    card: &'a CardSnapshot,
    background_color: Color,
}

impl<'a> CardTileView<'a> {
    pub fn new(card: &'a CardSnapshot, background_color: Color) -> Self {
        Self {
            card,
            background_color,
        }
    }

    fn facing(&self) -> impl Iterator<Item = Direction> + '_ {
        (0..8)
            .filter(|&offset| self.card.arrows & (1 << offset) != 0)
            .map(|offset| Direction::try_from(offset).expect("offset is a valid direction"))
    }

    fn top_line(&self) -> String {
        let mut northwest = " ";
        let mut north = " ";
        let mut northeast = " ";

        self.facing().for_each(|direction| match direction {
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

        self.facing().for_each(|direction| match direction {
            Direction::W => west = "◀",
            Direction::E => east = "▶",
            _ => {}
        });

        format!(
            "│{}{:^CARD_CONTENT_SPACE$}{}│",
            west,
            format_stats(self.card),
            east
        )
    }

    fn bottom_line(&self) -> String {
        let mut southwest = " ";
        let mut south = " ";
        let mut southeast = " ";

        self.facing().for_each(|direction| match direction {
            Direction::SW => southwest = "◣",
            Direction::S => south = "▼",
            Direction::SE => southeast = "◢",
            _ => {}
        });

        format!("│{southwest}{south:^CARD_CONTENT_SPACE$}{southeast}│")
    }

    fn name_line(&self) -> String {
        format!("│ {:^CARD_CONTENT_SPACE$} │", self.card.name)
    }
}

fn format_stats(card: &CardSnapshot) -> String {
    let attack = card.stats.attack >> 4;
    let physical_defense = card.stats.phys_defense >> 4;
    let magical_defense = card.stats.mag_defense >> 4;

    format!(
        "{attack:X}{}{physical_defense:X}{magical_defense:X}",
        card.stats.battle_class
    )
}

impl<'a> AsciiView for CardTileView<'a> {
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
        .on(self.background_color)
        .with(FRONT_COLOR)
        .to_string()
    }
}
