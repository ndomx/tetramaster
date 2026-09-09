use crate::{
    models::{board::Board, played_card::PlayedCard, player::Player, tile::Tile}, ui::ascii::{
        ascii_view::AsciiView,
        constants::{CARD_HEIGHT, CARD_WIDTH},
        tile_block_view::TileBlockView,
        tile_card_view::TileCardView,
        tile_empty_view::TileEmptyView,
        tile_view::TileView,
    }, utils::constants::{BOARD_SIZE, PLAYER_COUNT},
};

pub struct BoardView<'a> {
    board: &'a Board,
    players: [&'a Player; 2],
}

impl<'a> BoardView<'a> {
    pub fn new(board: &'a Board, players: [&'a Player; PLAYER_COUNT]) -> Self {
        Self { board, players }
    }

    fn row_tiles(&self, line: usize) -> Vec<TileView<'a>> {
        let row = line / CARD_HEIGHT;
        let tiles = self.board.row(row);

        tiles
            .iter()
            .map(|t| match t {
                Tile::Empty => TileView::Empty(TileEmptyView {}),
                Tile::Block => TileView::Block(TileBlockView {}),
                Tile::Card(played_card) => TileView::Card(self.build_card_view(played_card)),
            })
            .collect()
    }

    fn build_card_view(&self, played_card: &PlayedCard) -> TileCardView<'a> {
        let player = self
            .players
            .iter()
            .find(|p| p.id == played_card.owner_id)
            .unwrap();

        let card = self.board.played_card(played_card.card_id).unwrap();

        TileCardView::new(card, &player.name)
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
