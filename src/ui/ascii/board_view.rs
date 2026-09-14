use crossterm::style::Color;

use crate::{
    models::{game::Game, tile::Tile, tile_card::TileCard},
    ui::ascii::{
        ascii_view::AsciiView,
        constants::{CARD_HEIGHT, CARD_WIDTH},
        tile_block_view::TileBlockView,
        tile_card_view::TileCardView,
        tile_empty_view::TileEmptyView,
        tile_view::TileView,
    },
    utils::constants::BOARD_SIZE,
};

pub struct BoardView<'a> {
    game: &'a Game<'a>,
}

impl<'a> BoardView<'a> {
    pub fn new(game: &'a Game) -> Self {
        Self { game }
    }

    fn row_tiles(&self, line: usize) -> Vec<TileView<'a>> {
        let row = line / CARD_HEIGHT;
        let tiles = self.game.board.row(row);

        tiles
            .iter()
            .map(|t| match t {
                Tile::Empty => TileView::Empty(TileEmptyView {}),
                Tile::Block => TileView::Block(TileBlockView {}),
                Tile::Card(played_card) => TileView::Card(self.build_card_view(played_card)),
            })
            .collect()
    }

    fn build_card_view(&self, played_card: &TileCard) -> TileCardView<'a> {
        let pc = self.game.find_placed(played_card.card_id).unwrap();
        let color = if played_card.owner_id == self.game.player.id {
            Color::Blue
        } else {
            Color::Red
        };

        TileCardView::new(&pc.card, color)
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
