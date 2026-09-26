use std::{
    io::{self, stdout},
    thread::sleep,
    time::Duration,
};

use tetramaster::{
    models::session::{GameSession, InteractionState},
    ui::terminal::Terminal,
    utils::random::GameRng,
};

fn main() -> io::Result<()> {
    let seed = rand::random::<u64>();
    let mut game_session = GameSession::new(GameRng::from_seed(seed));
    let mut terminal = Terminal::new(stdout());

    run_game(&mut game_session, &mut terminal)
}

fn run_game(game_session: &mut GameSession, terminal: &mut Terminal) -> io::Result<()> {
    let mut snapshot = game_session.snapshot();
    let mut interaction = game_session.interaction_state();

    terminal.render(&snapshot)?;

    loop {
        let update = match interaction {
            InteractionState::AwaitingPlayerAction => {
                let action = terminal.read_action(&snapshot)?;
                game_session.dispatch(action)
            }
            InteractionState::Advancing => game_session.advance(),
            InteractionState::Finished => return Ok(()),
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
