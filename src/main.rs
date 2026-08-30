use rand::seq::IndexedRandom;

use crate::{assets::cards::CARDS, models::card::Card};

mod assets;
mod models;

fn main() {
    let mut rng = rand::rng();

    let cards = CARDS.sample(&mut rng, 5);
    cards.for_each(|c| {
        let card = Card::new(c);
        println!("{}", card);
    });
}
