use crate::{assets::cards::FLAN, models::card::Card};

mod assets;
mod models;

fn main() {
    let card = Card::new(
        87,
        0x67,
        &FLAN,
    );

    println!("{}", card);
}
