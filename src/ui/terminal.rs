use std::{
    io::{self, Stdout, Write},
    str::FromStr,
};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::{
    models::{action::GameAction, card::Card, game::Game, position::Position},
    ui::ascii::{
        ascii_view::AsciiView, board_view::BoardView, hand_card_view::HandCardView,
        score_view::ScoreView,
    },
    utils::constants::BOARD_SIZE,
};

pub struct Terminal {
    stdout: Stdout,
}

impl Terminal {
    pub fn new(stdout: Stdout) -> Self {
        Self { stdout }
    }

    pub fn render(&mut self, game: &Game) -> io::Result<()> {
        self.clear()?;

        let board_view = BoardView::new(game);
        let score_view = ScoreView::new(game);

        board_view.render()?;
        score_view.render()?;
        self.render_hand(&game.player.hand)?;

        Ok(())
    }

    pub fn read_action(&mut self, game: &Game) -> io::Result<GameAction> {
        let hand = game.player_hand();

        self.prompt("select a card to play: ")?;
        let idx: usize = self.parse_input(|&v| v < hand.len())?;
        let card = &hand[idx];

        self.prompt("select a row to play card: ")?;
        let row: usize = self.parse_input(|&v| v < BOARD_SIZE)?;

        self.prompt("select a col to play card: ")?;
        let col: usize = self.parse_input(|&v| v < BOARD_SIZE)?;

        Ok(GameAction::new(card.id, Position::new(row, col)))
    }

    fn clear(&mut self) -> io::Result<()> {
        execute!(&mut self.stdout, Clear(ClearType::All), MoveTo(0, 0))
    }

    fn render_hand(&self, hand: &[Card]) -> io::Result<()> {
        let views: Vec<HandCardView<'_>> = hand.iter().map(HandCardView::new).collect();
        let height = views.first().map(|v| v.height()).unwrap_or(0);

        for line in 0..height {
            views.iter().for_each(|v| {
                print!("{} ", v.line(line));
            });
            println!();
        }

        Ok(())
    }

    fn prompt(&mut self, message: &str) -> io::Result<()> {
        print!("{}", message);
        self.stdout.flush()
    }

    fn read_input(&self) -> io::Result<String> {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .map_err(|_| io::Error::other("Failed to read input"))?;

        Ok(input.trim().to_string())
    }

    fn parse_input<T: FromStr>(&self, validator: impl Fn(&T) -> bool) -> io::Result<T> {
        loop {
            let input = self.read_input()?;
            let Some(parsed) = input.parse::<T>().ok().filter(|v| validator(v)) else {
                println!("invalid choice!");
                continue;
            };

            break Ok(parsed);
        }
    }
}
