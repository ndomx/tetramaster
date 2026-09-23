use crossterm::style::{Color, Stylize};

use crate::{
    models::session::Game,
    ui::ascii::{ascii_view::AsciiView, constants::CARD_WIDTH},
    utils::constants::BOARD_SIZE,
};

pub struct ScoreView<'a> {
    game: &'a Game,
}

impl<'a> ScoreView<'a> {
    pub fn new(game: &'a Game) -> Self {
        Self { game }
    }
}

impl<'a> AsciiView for ScoreView<'a> {
    fn width(&self) -> usize {
        BOARD_SIZE * CARD_WIDTH
    }

    fn height(&self) -> usize {
        1
    }

    fn line(&self, _line: usize) -> String {
        let player_score = self.game.player_score().to_string();
        let cpu_score = self.game.cpu_score().to_string();

        let plain = format!("Player {player_score} | {cpu_score} CPU");

        let padding = self.width().saturating_sub(plain.len()) / 2;

        format!(
            "{}Player {} | {} CPU",
            " ".repeat(padding),
            player_score.with(Color::Blue),
            cpu_score.with(Color::Red),
        )
    }
}
