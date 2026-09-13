use std::array;

use rand::{RngExt, rngs::ThreadRng, seq::IteratorRandom};

use crate::{
    models::{
        card::Card, direction::Direction, placed_card::PlacedCard, position::Position, tile::Tile,
        tile_card::TileCard,
    },
    ui::ascii::tile_empty_view,
    utils::{
        constants::{BOARD_SIZE, TILE_TOTAL},
        helpers::{idx2pos, pos2idx},
    },
};

pub struct Board {
    tiles: [Tile; TILE_TOTAL],
    placed_cards: Vec<PlacedCard>,
}

impl Board {
    pub fn build(density: f64, rng: &mut ThreadRng) -> Self {
        Self {
            tiles: array::from_fn(|_| match rng.random_bool(density) {
                true => Tile::Block,
                false => Tile::Empty,
            }),
            placed_cards: vec![],
        }
    }

    pub fn get(&self, pos: Position) -> Option<&Tile> {
        let idx = pos2idx(pos);
        idx.and_then(|i| self.tiles.get(i))
    }

    pub fn get_card(&self, pos: Position) -> Option<&Card> {
        pos2idx(pos)
            .and_then(|idx| self.tiles.get(idx))
            .and_then(|tile| match tile {
                Tile::Card(played) => Some(played.card_id),
                _ => None,
            })
            .and_then(|card_id| self.find_placed(card_id))
            .map(|pc| &pc.card)
    }

    pub fn last_played(&self) -> Option<&PlacedCard> {
        self.placed_cards.last()
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

    pub fn find_placed(&self, card_id: u64) -> Option<&PlacedCard> {
        self.placed_cards.iter().find(|pc| {
            return pc.card.id == card_id;
        })
    }

    pub fn facing_cards(&self, pos: Position, dirs: Vec<Direction>) -> Vec<&TileCard> {
        dirs.iter()
            .filter_map(|dir| self.get_relative(pos, dir))
            .filter_map(|tile| match tile {
                Tile::Card(played) => Some(played),
                _ => None,
            })
            .collect()
    }

    pub fn neighboring_enemies(
        &self,
        pos: Position,
        dirs: Vec<Direction>,
    ) -> Result<Vec<&PlacedCard>, String> {
        let owner_id = self.tile_owner(pos)?;

        let cards = dirs
            .iter()
            .filter_map(|dir| self.get_relative(pos, dir))
            .filter_map(|tile| match tile {
                Tile::Card(played) => Some(played),
                _ => None,
            })
            .filter(|played| played.owner_id != owner_id)
            .filter_map(|played| self.find_placed(played.card_id))
            .collect();

        Ok(cards)
    }

    pub fn place_card(
        &mut self,
        card: Card,
        target: Position,
        owner_id: u64,
    ) -> Result<(), String> {
        let Some(idx) = pos2idx(target) else {
            return Err(format!("invalid position {:?}", target));
        };

        self.tiles[idx] = Tile::Card(TileCard {
            owner_id,
            card_id: card.id,
        });

        self.placed_cards.push(PlacedCard::new(card, target));

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

    pub fn swap_owner(&mut self, pos: Position, owner_id: u64) -> Result<&Card, String> {
        let Some(idx) = pos2idx(pos) else {
            return Err(format!("invalid pos {:?}", pos));
        };

        let card_id = self
            .tiles
            .get(idx)
            .and_then(|t| match t {
                Tile::Card(tc) => Some(tc.card_id),
                _ => None,
            })
            .ok_or("Tile is not a card".to_string())?;

        self.tiles[idx] = Tile::Card(TileCard { owner_id, card_id });
        self.find_placed(card_id)
            .map(|pc| &pc.card)
            .ok_or("Unable to find card".to_string())
    }

    pub fn tile_owner(&self, pos: Position) -> Result<u64, String> {
        self.get(pos)
            .and_then(|t| match t {
                Tile::Card(tile_card) => Some(tile_card.owner_id),
                _ => None,
            })
            .ok_or("Invalid position or tile".to_string())
    }
}
