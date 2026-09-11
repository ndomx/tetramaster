use std::{
    io::{self, Write, stdout}, str::FromStr,
};

use crate::{
    models::{
        card::Card,
        game::Game,
        game_command::{GameTurnInput, GameTurnOutput},
        position::Position,
    }, ui::{ascii::{ascii_view::AsciiView, board_view::BoardView, hand_card_view::HandCardView}, terminal::Terminal}, utils::constants::BOARD_SIZE,
};

mod assets;
mod models;
mod ui;
mod utils;

fn main() {
    let mut rng = rand::rng();
    let mut game = Game::new(0.25, &mut rng);

    let mut game_input = GameTurnInput::Continue;
    loop {
        let game_output = game.run(game_input);
        game_input = match game_output {
            GameTurnOutput::RenderBoard => on_render_board(&game),
            GameTurnOutput::RenderHand => on_render_hand(&game),
            GameTurnOutput::SelectPosition => on_select_card(&game),
            GameTurnOutput::Continue => GameTurnInput::Continue,
        };
    }
}

fn on_render_hand(game: &Game) -> GameTurnInput {
    let hand = game.player_hand();

    render_hand(hand);
    GameTurnInput::Continue
}

fn on_render_board(game: &Game) -> GameTurnInput {
    render_board(game);
    GameTurnInput::Continue
}

fn on_select_card(game: &Game) -> GameTurnInput {
    let hand = game.player_hand();

    let mut terminal = Terminal::new(stdout());
    terminal.render(game).ok();

    prompt("Select card: ");
    let idx: usize = parse_input(|&v| v < hand.len());
    let card = &hand[idx];

    println!("selected {} ({})", card.name(), card.id);

    let row: usize = parse_input(|&v| v < BOARD_SIZE);
    let col: usize = parse_input(|&v| v < BOARD_SIZE);
    println!("pos = ({row}, {col})");

    GameTurnInput::PlaceCard {
        card_id: card.id,
        target: Position::new(row, col),
    }
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

fn render_board(game: &Game) {
    let view = BoardView::new(&game.board, [&game.player, &game.cpu]);

    println!("Board");
    for line in 0..view.height() {
        println!("{}", view.line(line));
    }
}

fn prompt(message: &str) {
    print!("{}", message);
    io::stdout().flush().unwrap();
}

fn read_input() -> String {
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}

fn parse_input<T: FromStr>(validator: impl Fn(&T) -> bool) -> T {
    return loop {
        let input = read_input();
        let Some(parsed) = input.parse::<T>().ok().filter(|v| validator(v)) else {
            println!("invalid choice!");
            continue;
        };

        break parsed;
    };
}
