use crate::{
    models::{board::Board, card::Card, game::Game, player::Player}, ui::ascii::{ascii_view::AsciiView, board_view::BoardView, hand_card_view::HandCardView},
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

    let players = &game.players;
    render_board(&game.board, players);
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

fn render_board(board: &Board, players: &[Player]) {
    let view = BoardView::new(board, players);
    for line in 0..view.height() {
        println!("{}", view.line(line));
    }
}
