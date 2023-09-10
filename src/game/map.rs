use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Map(Vec<MapLayer>);

impl Map {
    pub fn new(layers: Vec<MapLayer>) -> Self {
        Self(layers)
    }

    pub fn get(&self, i: usize) -> Result<&MapLayer, MapError> {
        if i >= self.0.len() {
            return Err(MapError::OutOfBounds);
        }

        return Ok(&self.0[i]);
    }
}

impl From<MapLayer> for Map {
    fn from(value: MapLayer) -> Self {
        Self::new(vec![value])
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MapLayer(Vec<Vec<Tile>>);

impl MapLayer {
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

    pub fn update(&mut self, i: usize, j: usize, tile: Tile) -> Result<(), MapError> {
        if i >= self.width() || j >= self.height() {
            return Err(MapError::OutOfBounds);
        }

        self.0[i][j] = tile;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Hash, Deserialize, Serialize)]
pub enum MapError {
    OutOfBounds,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Tile {
    surface: SurfaceTile,
    stack: Vec<ObjectTile>,
}

impl Tile {
    pub fn new(surface: SurfaceTile, stack: Vec<ObjectTile>) -> Self {
        Self { surface, stack }
    }

    pub fn empty(surface: SurfaceTile) -> Self {
        Self::new(surface, Vec::new())
    }

    pub fn empty_ground() -> Self {
        Self::empty(SurfaceTile::Ground)
    }

    pub fn wall() -> Self {
        Self::empty(SurfaceTile::Wall)
    }

    pub fn get_surface(&self) -> SurfaceTile {
        self.surface
    }

    pub fn get_stack(&self) -> &Vec<ObjectTile> {
        &self.stack
    }

    pub fn get_top_of_stack(&self) -> Option<ObjectTile> {
        self.stack.last().cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum SurfaceTile {
    Ground,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ObjectTile {
    Item,
    Trap,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_layer_width_and_height() {
        let grid = MapLayer::fill(4, 9, Tile::wall());
        assert_eq!(grid.width(), 4);
        assert_eq!(grid.height(), 9);
    }

    #[test]
    fn test_map_layer_fills() {
        assert_eq!(
            MapLayer::new(Vec::new()),
            MapLayer::fill(0, 0, Tile::wall())
        );

        assert_eq!(
            MapLayer::new(vec![
                vec![Tile::wall(), Tile::wall()],
                vec![Tile::wall(), Tile::wall()],
                vec![Tile::wall(), Tile::wall()],
                vec![Tile::wall(), Tile::wall()],
            ]),
            MapLayer::fill(4, 2, Tile::wall())
        );
    }
    #[test]
    fn test_map_layer_i_j_tile_vector() {
        let grid = MapLayer::fill(4, 9, Tile::wall());
        let v = grid.as_i_j_tile_vector();
        assert_eq!(v.len(), grid.size());

        assert_eq!(v[0], (0, 0, &Tile::wall()));
        assert_eq!(v.last(), Some(&(3, 8, &Tile::wall())));
    }
}
