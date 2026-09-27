use std::{
    fmt::{Display, Formatter},
    io::{self, Stdout, Write},
};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::{
    models::{
        core::{board::BoardSide, geometry::Position},
        session::{
            BoardTileSnapshot, CardSnapshot, CombatResult, ControlChangeReason, GameAction,
            GameError, GameEvent, GameResult, GameSnapshot, PlayerSide,
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
            self.prompt("Play <card> <row> <column> (example: 2 3 4): ")?;
            let input = self.read_input()?;
            let command = match parse_player_command(&input, snapshot.human_hand.len()) {
                Ok(command) => command,
                Err(error) => {
                    writeln!(self.stdout, "{error}")?;
                    continue;
                }
            };

            let card = &snapshot.human_hand[command.card_index];
            let position = Position::new(command.row, command.col);
            let action = GameAction::new(card.id, position);
            if snapshot.legal_actions.contains(&action) {
                return Ok(action);
            }

            let message = match snapshot.board.get(command.row * BOARD_SIZE + command.col) {
                Some(BoardTileSnapshot::Blocked) => "That board position is blocked.",
                Some(BoardTileSnapshot::Occupied { .. }) => "That board position is occupied.",
                _ => "That card cannot be played at that position.",
            };
            writeln!(self.stdout, "{message}")?;
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

    fn clear(&mut self) -> io::Result<()> {
        execute!(&mut self.stdout, Clear(ClearType::All), MoveTo(0, 0))
    }

    fn render_hand(&self, hand: &[CardSnapshot]) -> io::Result<()> {
        let views: Vec<CardTileView<'_>> = hand
            .iter()
            .map(|card| CardTileView::new(card, crossterm::style::Color::Blue))
            .collect();
        let height = views.first().map(|view| view.height()).unwrap_or(0);

        for line in 0..height {
            views.iter().for_each(|view| {
                print!("{} ", view.line(line));
            });
            println!();
        }

        for (index, view) in views.iter().enumerate() {
            print!(
                "{:^width$} ",
                format!("Card {}", index + 1),
                width = view.width()
            );
        }
        println!();

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayerCommand {
    card_index: usize,
    row: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerInputError {
    ExpectedThreeNumbers,
    CardOutOfRange { hand_len: usize },
    RowOutOfRange,
    ColumnOutOfRange,
}

impl Display for PlayerInputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedThreeNumbers => formatter.write_str(
                "Enter exactly three numbers: card, row, and column (for example: 2 3 4).",
            ),
            Self::CardOutOfRange { hand_len } => {
                write!(formatter, "Choose a card from 1 to {hand_len}.")
            }
            Self::RowOutOfRange => {
                write!(formatter, "Choose a row from 1 to {BOARD_SIZE}.")
            }
            Self::ColumnOutOfRange => {
                write!(formatter, "Choose a column from 1 to {BOARD_SIZE}.")
            }
        }
    }
}

fn parse_player_command(input: &str, hand_len: usize) -> Result<PlayerCommand, PlayerInputError> {
    let values = input
        .split_whitespace()
        .map(str::parse::<usize>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| PlayerInputError::ExpectedThreeNumbers)?;

    let [card, row, col] = values.as_slice() else {
        return Err(PlayerInputError::ExpectedThreeNumbers);
    };

    if !(1..=hand_len).contains(card) {
        return Err(PlayerInputError::CardOutOfRange { hand_len });
    }
    if !(1..=BOARD_SIZE).contains(row) {
        return Err(PlayerInputError::RowOutOfRange);
    }
    if !(1..=BOARD_SIZE).contains(col) {
        return Err(PlayerInputError::ColumnOutOfRange);
    }

    Ok(PlayerCommand {
        card_index: card - 1,
        row: row - 1,
        col: col - 1,
    })
}

fn side_name(side: PlayerSide) -> &'static str {
    match side {
        PlayerSide::Human => "Player",
        PlayerSide::Cpu => "CPU",
    }
}

fn controller_name(controller: BoardSide) -> &'static str {
    match controller {
        BoardSide::Blue => "Player",
        BoardSide::Red => "CPU",
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
            attack_power,
            defense_power,
            outcome,
        } => {
            let winner = match outcome {
                CombatResult::AttackerWon => "attacker",
                CombatResult::DefenderWon => "defender",
            };
            format!(
                "Combat between cards {attacker_id} ({attack_power}) and {defender_id} ({defense_power}): {winner} won."
            )
        }
        GameEvent::ControlChanged {
            card_id,
            previous_controller,
            new_controller,
            reason,
        } => {
            let reason = match reason {
                ControlChangeReason::DirectCapture => "direct capture",
                ControlChangeReason::CombatVictory => "combat victory",
                ControlChangeReason::CombatDefeat => "combat defeat",
                ControlChangeReason::Combo => "combo",
            };
            format!(
                "Card {card_id} changed from {} to {} by {reason}.",
                controller_name(*previous_controller),
                controller_name(*new_controller)
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
    fn player_command_uses_one_based_card_and_board_coordinates() {
        assert_eq!(
            parse_player_command("2 3 4", 5),
            Ok(PlayerCommand {
                card_index: 1,
                row: 2,
                col: 3,
            })
        );
    }

    #[test]
    fn player_command_accepts_flexible_whitespace() {
        assert_eq!(
            parse_player_command("  1\t4  2  ", 3),
            Ok(PlayerCommand {
                card_index: 0,
                row: 3,
                col: 1,
            })
        );
    }

    #[test]
    fn player_command_reports_specific_range_errors() {
        assert_eq!(
            parse_player_command("4 1 1", 3),
            Err(PlayerInputError::CardOutOfRange { hand_len: 3 })
        );
        assert_eq!(
            parse_player_command("1 0 1", 3),
            Err(PlayerInputError::RowOutOfRange)
        );
        assert_eq!(
            parse_player_command("1 1 5", 3),
            Err(PlayerInputError::ColumnOutOfRange)
        );
    }

    #[test]
    fn player_command_rejects_malformed_input() {
        assert_eq!(
            parse_player_command("1 two 3", 5),
            Err(PlayerInputError::ExpectedThreeNumbers)
        );
        assert_eq!(
            parse_player_command("1 2", 5),
            Err(PlayerInputError::ExpectedThreeNumbers)
        );
        assert_eq!(
            parse_player_command("1 2 3 4", 5),
            Err(PlayerInputError::ExpectedThreeNumbers)
        );
    }

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
