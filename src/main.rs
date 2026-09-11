use std::io::stdout;

use crate::{models::game::Game, ui::terminal::Terminal};

mod assets;
mod models;
mod ui;
mod utils;

fn main() {
    let mut rng = rand::rng();
    let mut game = Game::new(0.25, &mut rng);

    let mut terminal = Terminal::new(stdout());

    loop {
        terminal.render(&game).ok();

        game.run().ok();

        let action = terminal.read_action(&game).unwrap();
        game.play_card(action).ok();
    }
}
