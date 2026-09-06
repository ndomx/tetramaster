use crate::ui::ascii::{
    ascii_view::AsciiView,
    constants::{CARD_HEIGHT, CARD_WIDTH},
    tile_block_view::TileBlockView,
    tile_card_view::TileCardView,
    tile_empty_view::TileEmptyView,
};

pub enum TileView<'a> {
    Empty(TileEmptyView),
    Block(TileBlockView),
    Card(TileCardView<'a>),
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
