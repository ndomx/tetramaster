use crate::models::game::Game;

mod assets;
mod models;
mod utils;

fn main() {
    let mut rng = rand::rng();

    let _ = Game::new(0.65, &mut rng);
}
