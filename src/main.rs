use std::{
    io::{self, stdout},
    thread::sleep,
    time::Duration,
};

use tetramaster::{
    models::session::{Game, InteractionState},
    ui::terminal::Terminal,
    utils::random::GameRng,
};

fn main() -> io::Result<()> {
    let seed = rand::random::<u64>();
    println!("Game seed: {seed}");
    let mut game = Game::new(0.25, GameRng::from_seed(seed));
    let mut terminal = Terminal::new(stdout());

    run_game(&mut game, &mut terminal)
}

fn run_game(game: &mut Game, terminal: &mut Terminal) -> io::Result<()> {
    let mut snapshot = game.snapshot();
    let mut interaction = game.interaction_state();

    terminal.render(&snapshot)?;

    loop {
        let update = match interaction {
            InteractionState::AwaitingPlayerAction => {
                let action = terminal.read_action(&snapshot)?;
                game.dispatch(action)
            }
            InteractionState::Advancing => game.advance(),
            InteractionState::Finished => {
                terminal.render_result(snapshot.result)?;
                return Ok(());
            }
        };

        match update {
            Ok(update) => {
                snapshot = update.snapshot;
                interaction = update.interaction;
                terminal.render(&snapshot)?;
                for event in &update.events {
                    terminal.render_event(event)?;
                }
            }
            Err(error) => {
                terminal.render_error(&error)?;
            }
        }

        sleep(Duration::from_millis(500));
    }
}
