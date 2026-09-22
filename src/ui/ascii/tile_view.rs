use crate::ui::ascii::{
    ascii_view::AsciiView,
    blocked_tile_view::BlockedTileView,
    card_tile_view::CardTileView,
    constants::{CARD_HEIGHT, CARD_WIDTH},
    empty_tile_view::EmptyTileView,
};

pub enum TileView<'a> {
    Empty(EmptyTileView),
    Block(BlockedTileView),
    Card(CardTileView<'a>),
}

impl<'a> AsciiView for TileView<'a> {
    fn width(&self) -> usize {
        CARD_WIDTH
    }

    fn height(&self) -> usize {
        CARD_HEIGHT
    }

    fn line(&self, line: usize) -> String {
        match self {
            TileView::Empty(view) => view.line(line),
            TileView::Block(view) => view.line(line),
            TileView::Card(view) => view.line(line),
        }
    }
}
