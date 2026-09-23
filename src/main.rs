use std::{io::stdout, thread::sleep, time::Duration};

use tetramaster::{
    models::session::{Game, GamePhase},
    ui::terminal::Terminal,
    utils::random::GameRng,
};

fn main() {
    let seed = rand::random::<u64>();
    println!("Game seed: {seed}");
    let mut game = Game::new(0.25, GameRng::from_seed(seed));

    let mut terminal = Terminal::new(stdout());

    while game.state != GamePhase::Finished {
        sleep(Duration::from_millis(500));

        terminal.render(&game).ok();
        println!("Current State = {:?}", game.state);

        let _ = if game.awaiting_input() {
            let action = terminal.read_action(&game).unwrap();
            game.play_card(action)
        } else {
            game.run()
        };
    }

    let player_score = game.player_score();
    let cpu_score = game.cpu_score();

    let winner = match player_score > cpu_score {
        true => game.player,
        false => game.cpu,
    };

    println!("Winner: {} !!", { winner.name });
}
