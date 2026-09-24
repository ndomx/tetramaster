use std::fmt::{Display, Formatter};

use crate::models::core::{
    card::{Card, CardStats},
    geometry::Position,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSide {
    Human,
    Cpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionState {
    AwaitingPlayerAction,
    Advancing,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    NotStarted,
    Turn,
    Resolving,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Winner(PlayerSide),
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResult {
    AttackerWon,
    DefenderWon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipChangeReason {
    DirectCapture,
    CombatVictory,
    CombatDefeat,
    Combo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSnapshot {
    pub id: u64,
    pub definition_index: usize,
    pub name: String,
    pub arrows: u8,
    pub stats: CardStats,
}

impl From<&Card> for CardSnapshot {
    fn from(card: &Card) -> Self {
        Self {
            id: card.id,
            definition_index: card.asset.index,
            name: card.asset.name.clone(),
            arrows: card.arrows,
            stats: card.stats.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardTileSnapshot {
    Empty,
    Blocked,
    Occupied {
        owner: PlayerSide,
        card: CardSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSnapshot {
    pub board: Vec<BoardTileSnapshot>,
    pub human_hand: Vec<CardSnapshot>,
    pub cpu_hand_count: usize,
    pub human_score: usize,
    pub cpu_score: usize,
    pub active_player: Option<PlayerSide>,
    pub phase: SessionPhase,
    pub result: Option<GameResult>,
    pub legal_actions: Vec<super::GameAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    GameStarted {
        first_player: PlayerSide,
    },
    TurnStarted {
        player: PlayerSide,
    },
    CardPlaced {
        player: PlayerSide,
        card_id: u64,
        position: Position,
    },
    CombatResolved {
        attacker_id: u64,
        defender_id: u64,
        outcome: CombatResult,
    },
    OwnershipChanged {
        card_id: u64,
        previous_owner: PlayerSide,
        new_owner: PlayerSide,
        reason: OwnershipChangeReason,
    },
    TurnEnded {
        player: PlayerSide,
    },
    GameFinished {
        result: GameResult,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameUpdate {
    pub events: Vec<GameEvent>,
    pub snapshot: GameSnapshot,
    pub interaction: InteractionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    InvalidInteraction {
        expected: InteractionState,
        actual: InteractionState,
    },
    IllegalPosition(Position),
    CardNotInHand(u64),
    Internal(String),
}

impl Display for GameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInteraction { expected, actual } => {
                write!(formatter, "expected {expected:?}, found {actual:?}")
            }
            Self::IllegalPosition(position) => write!(formatter, "illegal position {position:?}"),
            Self::CardNotInHand(card_id) => write!(formatter, "card {card_id} is not in hand"),
            Self::Internal(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for GameError {}

impl From<String> for GameError {
    fn from(message: String) -> Self {
        Self::Internal(message)
    }
}

impl From<GameError> for String {
    fn from(error: GameError) -> Self {
        error.to_string()
    }
}
