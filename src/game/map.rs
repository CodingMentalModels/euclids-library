use std::ops::{Add, AddAssign};

use bevy::prelude::*;
use bevy::utils::Duration;
use serde::{Deserialize, Serialize};

use super::{
    events::Direction,
    particle::{
        ParticleAppearance, ParticleDirection, ParticleDuration, ParticleLocation,
        ParticleMovement, ParticleSpec, ParticleTiming,
    },
    ui_state::{AsciiTileAppearance, ColorCode},
};

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Map(Vec<MapLayer>);

impl Map {
    pub fn new(layers: Vec<MapLayer>) -> Self {
        Self(layers)
    }

    pub fn get_layer(&self, i: usize) -> Result<&MapLayer, MapError> {
        if i >= self.0.len() {
            return Err(MapError::OutOfBounds);
        }
        return Ok(&self.0[i]);
    }

    pub fn get(&self, location: MapLocation) -> Result<&Tile, MapError> {
        self.get_layer(location.map_layer)
            .and_then(|layer| layer.get_from_location(location.get_tile_location()))
    }
}

impl From<MapLayer> for Map {
    fn from(value: MapLayer) -> Self {
        Self::new(vec![value])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MapLayer(Vec<Vec<Tile>>);

impl MapLayer {
    pub fn new(tiles: Vec<Vec<Tile>>) -> Self {
        if tiles.len() > 0 {
            let height = tiles[0].len();
            assert!(tiles.iter().all(|column| column.len() == height));
        }
        Self(tiles)
    }

    pub fn get_tiles_cloned(&self) -> Vec<Vec<Tile>> {
        self.0.clone()
    }

    pub fn fill(width: usize, height: usize, tile: Tile) -> Self {
        Self::new(vec![vec![tile; height]; width])
    }

    pub fn get(&self, i: usize, j: usize) -> Result<&Tile, MapError> {
        if !self.is_in_bounds(i, j) {
            Err(MapError::OutOfBounds)
        } else {
            Ok(&self.0[i][j])
        }
    }

    pub fn get_from_location(&self, location: TileLocation) -> Result<&Tile, MapError> {
        self.get(location.x() as usize, location.y() as usize)
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

    pub fn as_location_and_tile_vector(&self) -> Vec<(TileLocation, &Tile)> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, column)| {
                column
                    .iter()
                    .enumerate()
                    .map(|(j, tile)| (TileLocation::new(i as i32, j as i32), tile))
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

    pub fn update_edges(&mut self, tile: &Tile) {
        let width = self.width();
        let height = self.height();
        if width == 0 || height == 0 {
            return;
        }
        // Top & Bottom
        (0..width).into_iter().for_each(|i| {
            self.0[i][0] = tile.clone();
            self.0[i][height - 1] = tile.clone();
        });

        // Left & Right
        (0..height).into_iter().for_each(|j| {
            self.0[0][j] = tile.clone();
            self.0[width - 1][j] = tile.clone();
        })
    }

    fn is_in_bounds(&self, i: usize, j: usize) -> bool {
        (i >= self.width()) || (j >= self.height())
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

    pub fn object(object: ObjectTile) -> Self {
        Self::new(SurfaceTile::Ground, vec![object])
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
    Fireplace,
}

impl SurfaceTile {
    pub fn get_particle_spec(&self) -> Option<ParticleSpec> {
        match self {
            Self::Fireplace => Some(ParticleSpec::new(
                ParticleTiming::Every(ParticleDuration::Exact(Duration::from_millis(500))),
                ParticleLocation::Exact(Direction::Up.as_tile_location()),
                ParticleMovement::new(
                    ParticleTiming::Every(ParticleDuration::Exact(Duration::from_millis(500))),
                    ParticleDirection::Weighted(vec![
                        (Direction::Up, 2),
                        (Direction::UpLeft, 1),
                        (Direction::UpRight, 1),
                    ]),
                ),
                ParticleAppearance::Constant(AsciiTileAppearance::new('S', ColorCode::Gray)),
            )),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ObjectTile {
    Item,
    Trap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MapLocation {
    map_layer: usize,
    tile_location: TileLocation,
}

impl MapLocation {
    pub fn new(map_layer: usize, tile_location: TileLocation) -> Self {
        Self {
            map_layer,
            tile_location,
        }
    }

    pub fn get_map_layer(&self) -> usize {
        self.map_layer
    }

    pub fn get_tile_location(&self) -> TileLocation {
        self.tile_location
    }

    pub fn translate(&mut self, amount: TileLocation) {
        self.tile_location += amount;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TileLocation {
    pub i: i32,
    pub j: i32,
}

impl Add for TileLocation {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.i + rhs.i, self.j + rhs.j)
    }
}

impl AddAssign for TileLocation {
    fn add_assign(&mut self, rhs: Self) {
        self.i = self.i + rhs.i;
        self.j = self.j + rhs.j;
    }
}

impl TileLocation {
    pub fn new(i: i32, j: i32) -> Self {
        Self { i, j }
    }

    pub fn x(&self) -> i32 {
        self.i
    }

    pub fn y(&self) -> i32 {
        self.j
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn step(&mut self, direction: Direction) {
        let new_self = self.clone() + direction.as_tile_location();
        self.i = new_self.i;
        self.j = new_self.j;
    }
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
        let v = grid.as_location_and_tile_vector();
        assert_eq!(v.len(), grid.size());

        assert_eq!(v[0], (TileLocation::zero(), &Tile::wall()));
        assert_eq!(v.last(), Some(&(TileLocation::new(3, 8), &Tile::wall())));
    }
}
