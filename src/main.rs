use crate::{
    models::{card::Card, game::Game},
    ui::ascii::{ascii_view::AsciiView, hand_card_view::HandCardView},
};

mod assets;
mod models;
mod ui;
mod utils;

fn main() {
    let mut rng = rand::rng();

    let game = Game::new(0.65, &mut rng);
    let hand = &game.players[0].hand;
    render_hand(hand);
}

fn render_hand(hand: &Vec<Card>) {
    let views: Vec<HandCardView<'_>> = hand.iter().map(|c| HandCardView::new(c)).collect();
    let height = views
        .first()
        .and_then(|v| Some(v.height()))
        .or(Some(0))
        .unwrap();

    for line in 0..height {
        views.iter().for_each(|v| {
            print!("{} ", v.line(line));
        });
        println!();
    }
}
