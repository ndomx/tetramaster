use std::io::{self, Stdout};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::{
    models::{card::Card, game::Game},
    ui::ascii::{ascii_view::AsciiView, board_view::BoardView, hand_card_view::HandCardView},
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

        let board = &game.board;
        let player = &game.player;
        let cpu = &game.cpu;

        let board_view = BoardView::new(board, [player, cpu]);

        board_view.render()?;
        self.render_hand(&player.hand)?;

        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        execute!(&mut self.stdout, Clear(ClearType::All), MoveTo(0, 0))
    }

    fn render_hand(&self, hand: &Vec<Card>) -> io::Result<()> {
        let views: Vec<HandCardView<'_>> = hand.iter().map(|c| HandCardView::new(c)).collect();
        let height = views
            .first()
            .and_then(|v| Some(v.height()))
            .or(Some(0))
            .unwrap();

        for line in 0..height {
            views.iter().for_each(|v| {
                print!("{} ", v.line(line));
            });
            println!();
        }

        Ok(())
    }
}
