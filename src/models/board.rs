use std::array;

use rand::{RngExt, rngs::ThreadRng, seq::IteratorRandom};

use crate::{
    models::{
        card::Card, direction::Direction, played_card::PlayedCard, position::Position, tile::Tile,
    },
    utils::{
        constants::{BOARD_SIZE, TILE_TOTAL},
        helpers::{idx2pos, pos2idx},
    },
};

pub struct Board {
    tiles: [Tile; TILE_TOTAL],
    played_cards: Vec<Card>,
}

impl Board {
    pub fn build(density: f64, rng: &mut ThreadRng) -> Self {
        Self {
            tiles: array::from_fn(|_| match rng.random_bool(density) {
                true => Tile::Block,
                false => Tile::Empty,
            }),
            played_cards: vec![],
        }
    }

    pub fn get(&self, pos: Position) -> Option<&Tile> {
        let idx = pos2idx(pos);
        idx.and_then(|i| self.tiles.get(i))
    }

    pub fn get_relative(&self, pos: Position, dir: &Direction) -> Option<&Tile> {
        let next = pos.relative(
            dir,
            Position {
                row: BOARD_SIZE,
                col: BOARD_SIZE,
            },
        );

        next.and_then(|p| self.get(p))
    }

    pub fn row(&self, row: usize) -> &[Tile] {
        let lower = BOARD_SIZE * row;
        let upper = lower + BOARD_SIZE;
        self.tiles.get(lower..upper).unwrap_or_default()
    }

    pub fn played_card(&self, card_id: u64) -> Option<&Card> {
        self.played_cards.iter().find(|c| {
            return c.id == card_id;
        })
    }

    pub fn neighboring_enemies(
        &self,
        pos: Position,
        dirs: Vec<Direction>,
        player_id: u64,
    ) -> Vec<&Card> {
        dirs.iter()
            .filter_map(|dir| self.get_relative(pos, dir))
            .filter_map(|tile| match tile {
                Tile::Card(played) => Some(played),
                _ => None,
            })
            .filter(|played| played.owner_id != player_id)
            .filter_map(|played| self.find_card(played.card_id))
            .collect()
    }

    pub fn place_card(&mut self, card: Card, target: Position, owner_id: u64) -> Result<(), ()> {
        let Some(idx) = pos2idx(target) else {
            return Err(());
        };

        self.tiles[idx] = Tile::Card(PlayedCard {
            owner_id,
            card_id: card.id,
        });

        self.played_cards.push(card);

        Ok(())
    }

    pub fn count_available(&self) -> usize {
        self.tiles
            .iter()
            .filter(|tile| tile == &&Tile::Empty)
            .count()
    }

    pub fn is_available(&self, pos: Position) -> bool {
        self.get(pos) == Some(&Tile::Empty)
    }

    pub fn find_availables(&self) -> Vec<Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| match tile {
                &Tile::Empty => idx2pos(idx),
                _ => None,
            })
            .collect()
    }

    pub fn find_available(&self, rng: &mut ThreadRng) -> Option<Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| match tile {
                Tile::Empty => idx2pos(idx),
                _ => None,
            })
            .choose(rng)
    }

    fn find_card(&self, card_id: u64) -> Option<&Card> {
        self.played_cards.iter().find(|c| c.id == card_id)
    }
}
