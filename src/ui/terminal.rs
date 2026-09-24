use std::{
    io::{self, Stdout, Write},
    str::FromStr,
};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::{
    models::{
        core::geometry::Position,
        session::{
            CardSnapshot, CombatResult, GameAction, GameError, GameEvent, GameResult, GameSnapshot,
            OwnershipChangeReason, PlayerSide,
        },
    },
    ui::ascii::{
        ascii_view::AsciiView, board_view::BoardView, card_tile_view::CardTileView,
        score_view::ScoreView,
    },
    utils::constants::BOARD_SIZE,
};

pub struct Terminal {
    stdout: Stdout,
}

impl Terminal {
    pub fn new(stdout: Stdout) -> Self {
        Self { stdout }
    }

    pub fn render(&mut self, snapshot: &GameSnapshot) -> io::Result<()> {
        self.clear()?;

        let board_view = BoardView::new(snapshot);
        let score_view = ScoreView::new(snapshot);

        board_view.render()?;
        score_view.render()?;
        self.render_hand(&snapshot.human_hand)?;

        Ok(())
    }

    pub fn read_action(&mut self, snapshot: &GameSnapshot) -> io::Result<GameAction> {
        loop {
            self.prompt("select a card to play: ")?;
            let idx: usize = self.parse_input(|&v| v < snapshot.human_hand.len())?;
            let card = &snapshot.human_hand[idx];

            self.prompt("select a row to play card: ")?;
            let row: usize = self.parse_input(|&v| v < BOARD_SIZE)?;

            self.prompt("select a col to play card: ")?;
            let col: usize = self.parse_input(|&v| v < BOARD_SIZE)?;

            let action = GameAction::new(card.id, Position::new(row, col));
            if snapshot.legal_actions.contains(&action) {
                return Ok(action);
            }

            println!("that card cannot be played at that position");
        }
    }

    pub fn render_event(&mut self, event: &GameEvent) -> io::Result<()> {
        println!("{}", event_message(event));
        self.stdout.flush()
    }

    pub fn render_error(&mut self, error: &GameError) -> io::Result<()> {
        println!("Unable to continue: {error}. Please try again.");
        self.stdout.flush()
    }

    pub fn render_result(&mut self, result: Option<GameResult>) -> io::Result<()> {
        let message = match result {
            Some(GameResult::Winner(PlayerSide::Human)) => "Winner: Player!!",
            Some(GameResult::Winner(PlayerSide::Cpu)) => "Winner: CPU!!",
            Some(GameResult::Draw) => "The game is a draw!",
            None => "The game ended without a result.",
        };
        println!("{message}");
        self.stdout.flush()
    }

    fn clear(&mut self) -> io::Result<()> {
        execute!(&mut self.stdout, Clear(ClearType::All), MoveTo(0, 0))
    }

    fn render_hand(&self, hand: &[CardSnapshot]) -> io::Result<()> {
        let views: Vec<CardTileView<'_>> = hand
            .iter()
            .map(|card| CardTileView::new(card, crossterm::style::Color::Blue))
            .collect();
        let height = views.first().map(|v| v.height()).unwrap_or(0);

        for line in 0..height {
            views.iter().for_each(|v| {
                print!("{} ", v.line(line));
            });
            println!();
        }

        Ok(())
    }

    fn prompt(&mut self, message: &str) -> io::Result<()> {
        print!("{}", message);
        self.stdout.flush()
    }

    fn read_input(&self) -> io::Result<String> {
        let mut input = String::new();

        let bytes_read = io::stdin().read_line(&mut input)?;
        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "terminal input closed",
            ));
        }

        Ok(input.trim().to_string())
    }

    fn parse_input<T: FromStr>(&self, validator: impl Fn(&T) -> bool) -> io::Result<T> {
        loop {
            let input = self.read_input()?;
            let Some(parsed) = input.parse::<T>().ok().filter(|v| validator(v)) else {
                println!("invalid choice!");
                continue;
            };

            break Ok(parsed);
        }
    }
}

fn side_name(side: PlayerSide) -> &'static str {
    match side {
        PlayerSide::Human => "Player",
        PlayerSide::Cpu => "CPU",
    }
}

fn event_message(event: &GameEvent) -> String {
    match event {
        GameEvent::GameStarted { first_player } => {
            format!("Game started. {} goes first.", side_name(*first_player))
        }
        GameEvent::TurnStarted { player } => format!("{} turn.", side_name(*player)),
        GameEvent::CardPlaced {
            player,
            card_id,
            position,
        } => format!(
            "{} placed card {card_id} at row {}, column {}.",
            side_name(*player),
            position.row,
            position.col
        ),
        GameEvent::CombatResolved {
            attacker_id,
            defender_id,
            outcome,
        } => {
            let winner = match outcome {
                CombatResult::AttackerWon => "attacker",
                CombatResult::DefenderWon => "defender",
            };
            format!("Combat between cards {attacker_id} and {defender_id}: {winner} won.")
        }
        GameEvent::OwnershipChanged {
            card_id,
            previous_owner,
            new_owner,
            reason,
        } => {
            let reason = match reason {
                OwnershipChangeReason::DirectCapture => "direct capture",
                OwnershipChangeReason::CombatVictory => "combat victory",
                OwnershipChangeReason::CombatDefeat => "combat defeat",
                OwnershipChangeReason::Combo => "combo",
            };
            format!(
                "Card {card_id} changed from {} to {} by {reason}.",
                side_name(*previous_owner),
                side_name(*new_owner)
            )
        }
        GameEvent::TurnEnded { player } => format!("{} turn ended.", side_name(*player)),
        GameEvent::GameFinished { result } => match result {
            GameResult::Winner(side) => format!("Game finished. {} won.", side_name(*side)),
            GameResult::Draw => "Game finished in a draw.".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_messages_cover_player_visible_contract_events() {
        assert_eq!(
            event_message(&GameEvent::TurnStarted {
                player: PlayerSide::Cpu,
            }),
            "CPU turn."
        );
        assert_eq!(
            event_message(&GameEvent::GameFinished {
                result: GameResult::Draw,
            }),
            "Game finished in a draw."
        );
    }
}
