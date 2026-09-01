use crate::{assets::cards::CARDS, models::{card::Card, game::Game}, ui::ascii::{ascii_view::AsciiView, hand_card_view::HandCardView}};

mod assets;
mod models;
mod ui;
mod utils;

fn main() {
    let mut rng = rand::rng();

    let _ = Game::new(0.65, &mut rng);

    let a = &CARDS[0];
    let c = Card::new(a);
    let renderer = HandCardView::new(&c);

    for i in 0..renderer.height() {
        println!("{}", renderer.line(i));
    }
}
