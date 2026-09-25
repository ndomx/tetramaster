use crossterm::style::Color;

use crate::{
    models::{
        core::board::BoardSide,
        session::{BoardTileSnapshot, CardSnapshot, GameSnapshot},
    },
    ui::ascii::{
        ascii_view::AsciiView,
        blocked_tile_view::BlockedTileView,
        card_tile_view::CardTileView,
        constants::{CARD_HEIGHT, CARD_WIDTH},
        empty_tile_view::EmptyTileView,
        tile_view::TileView,
    },
    utils::constants::BOARD_SIZE,
};

pub struct BoardView<'a> {
    snapshot: &'a GameSnapshot,
}

impl<'a> BoardView<'a> {
    pub fn new(snapshot: &'a GameSnapshot) -> Self {
        Self { snapshot }
    }

    fn row_tiles(&self, line: usize) -> Vec<TileView<'a>> {
        let row = line / CARD_HEIGHT;
        let start = row * BOARD_SIZE;
        let tiles = &self.snapshot.board[start..start + BOARD_SIZE];

        tiles
            .iter()
            .map(|tile| match tile {
                BoardTileSnapshot::Empty => TileView::Empty(EmptyTileView {}),
                BoardTileSnapshot::Blocked => TileView::Blocked(BlockedTileView {}),
                BoardTileSnapshot::Occupied { controller, card } => {
                    TileView::Card(self.build_card_view(*controller, card))
                }
            })
            .collect()
    }

    fn build_card_view(&self, controller: BoardSide, card: &'a CardSnapshot) -> CardTileView<'a> {
        let color = match controller {
            BoardSide::Blue => Color::Blue,
            BoardSide::Red => Color::Red,
        };

        CardTileView::new(card, color)
    }
}

impl<'a> AsciiView for BoardView<'a> {
    fn width(&self) -> usize {
        BOARD_SIZE * CARD_WIDTH
    }

    fn height(&self) -> usize {
        BOARD_SIZE * CARD_HEIGHT
    }

    fn line(&self, line: usize) -> String {
        let row_views = self.row_tiles(line);
        let strings: Vec<String> = row_views
            .iter()
            .map(|view| view.line(line % view.height()))
            .collect();

        strings.join("")
    }
}
