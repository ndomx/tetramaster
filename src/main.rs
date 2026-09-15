use std::{io::stdout, thread::sleep, time::Duration};

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
        sleep(Duration::from_millis(1000));
        // sleep(Duration::from_millis(250));
        terminal.render(&game).ok();
        println!("Current State = {:?}", game.state);

        let _ = if game.awaiting_input() {
            let action = terminal.read_action(&game).unwrap();
            game.play_card(action)
        } else {
            game.run()
        };
    }
}
