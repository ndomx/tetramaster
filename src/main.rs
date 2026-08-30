use crate::models::game::Game;

mod assets;
mod constants;
mod models;

fn main() {
    let mut rng = rand::rng();

    let _ = Game::new(0.65, &mut rng);
}
