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

    pub fn size(&self) -> usize {
        self.width() * self.height()
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

    pub fn as_i_j_tile_vector(&self) -> Vec<(usize, usize, &Tile)> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, column)| {
                column
                    .iter()
                    .enumerate()
                    .map(|(j, tile)| (i, j, tile))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    pub fn update(&mut self, i: usize, j: usize, tile: Tile) -> Result<(), TileGridError> {
        if i >= self.width() || j >= self.height() {
            return Err(TileGridError::OutOfBounds);
        }

        self.0[i][j] = tile;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TileGridError {
    OutOfBounds,
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
    #[test]
    fn test_tilegrid_i_j_tile_vector() {
        let grid = TileGrid::fill(4, 9, Tile::Ascii('a'));
        let v = grid.as_i_j_tile_vector();
        assert_eq!(v.len(), grid.size());

        assert_eq!(v[0], (0, 0, &Tile::Ascii('a')));
        assert_eq!(v.last(), Some(&(3, 8, &Tile::Ascii('a'))));
    }
}
