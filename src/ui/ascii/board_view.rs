use crossterm::style::Color;

use crate::{
    models::session::{BoardTileSnapshot, CardSnapshot, GameSnapshot, PlayerSide},
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
            .map(|t| match t {
                BoardTileSnapshot::Empty => TileView::Empty(EmptyTileView {}),
                BoardTileSnapshot::Blocked => TileView::Block(BlockedTileView {}),
                BoardTileSnapshot::Occupied { owner, card } => {
                    TileView::Card(self.build_card_view(*owner, card))
                }
            })
            .collect()
    }

    fn build_card_view(&self, owner: PlayerSide, card: &'a CardSnapshot) -> CardTileView<'a> {
        let color = match owner {
            PlayerSide::Human => Color::Blue,
            PlayerSide::Cpu => Color::Red,
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
