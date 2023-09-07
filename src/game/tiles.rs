use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Resource, PartialEq, Eq)]
pub struct TileGrid(Vec<Vec<Tile>>);

impl TileGrid {
    pub fn new(tiles: Vec<Vec<Tile>>) -> Self {
        if tiles.len() > 0 {
            let height = tiles[0].len();
            assert!(tiles.iter().all(|column| column.len() == height));
        }
        Self(tiles)
    }

    pub fn fill(width: usize, height: usize, tile: Tile) -> Self {
        Self::new(vec![vec![tile; height]; width])
    }

    pub fn get(&self, i: usize, j: usize) -> &Tile {
        &self.0[i][j]
    }

    pub fn width(&self) -> usize {
        self.0.len()
    }

    pub fn height(&self) -> usize {
        if self.0.len() == 0 {
            0
        } else {
            self.0[0].len()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tile {
    Ascii(char),
    Sprite(Handle<TextureAtlas>),
}

impl Tile {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilegrid_width_and_height() {
        let grid = TileGrid::fill(4, 9, Tile::Ascii('a'));
        assert_eq!(grid.width(), 4);
        assert_eq!(grid.height(), 9);
    }

    #[test]
    fn test_tilegrid_fills() {
        assert_eq!(
            TileGrid::new(Vec::new()),
            TileGrid::fill(0, 0, Tile::Ascii('a'))
        );

        assert_eq!(
            TileGrid::new(vec![
                vec![Tile::Ascii('a'), Tile::Ascii('a')],
                vec![Tile::Ascii('a'), Tile::Ascii('a')],
                vec![Tile::Ascii('a'), Tile::Ascii('a')],
                vec![Tile::Ascii('a'), Tile::Ascii('a')],
            ]),
            TileGrid::fill(4, 2, Tile::Ascii('a'))
        );
    }
}
