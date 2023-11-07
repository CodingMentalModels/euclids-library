use std::ops::{Add, AddAssign};

use bevy::utils::Duration;
use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

use super::events::BoundStateComponent;
use super::resources::GameState;
use super::{
    events::Direction,
    particle::{
        ParticleAppearance, ParticleDirection, ParticleDuration, ParticleLocation,
        ParticleMovement, ParticleSpec, ParticleTiming,
    },
};
use crate::constants::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
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

    pub fn is_traversable(&self, location: MapLocation) -> Result<bool, MapError> {
        self.get_layer(location.map_layer)
            .and_then(|layer| layer.is_traversable(location.get_tile_location()))
    }

    pub fn update(&mut self, tile: Tile, location: MapLocation) -> Result<(), MapError> {
        self.0[location.get_map_layer()].update_at_location(tile, location.get_tile_location())
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

    pub fn update_at_location(
        &mut self,
        tile: Tile,
        location: TileLocation,
    ) -> Result<(), MapError> {
        self.update(tile, location.i as usize, location.j as usize)
    }

    pub fn update(&mut self, tile: Tile, i: usize, j: usize) -> Result<(), MapError> {
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
        !self.is_out_of_bounds(i, j)
    }

    fn is_out_of_bounds(&self, i: usize, j: usize) -> bool {
        (i >= self.width()) || (j >= self.height())
    }

    fn is_traversable(&self, tile_location: TileLocation) -> Result<bool, MapError> {
        Ok(self.get_from_location(tile_location)?.is_traversable())
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

    pub fn is_traversable(&self) -> bool {
        match self.get_surface() {
            SurfaceTile::Ground => true,
            SurfaceTile::Fireplace => true,
            SurfaceTile::Wall => false,
        }
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

impl From<TileLocation> for MapLocation {
    fn from(value: TileLocation) -> Self {
        Self::new(0, value)
    }
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct TileGrid {
    grid: Vec<Vec<TileAppearance>>,
}

impl TileGrid {
    pub fn new(grid: Vec<Vec<TileAppearance>>) -> Self {
        Self { grid }
    }

    pub fn from_map_layer(layer: MapLayer) -> Self {
        Self::new(
            layer
                .get_tiles_cloned()
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|tile| TileAppearance::from_tile(&tile))
                        .collect()
                })
                .collect(),
        )
    }

    pub fn get(&self, i: usize, j: usize) -> Option<TileAppearance> {
        if self.is_in_bounds(i, j) {
            Some(self.grid[i][j].clone())
        } else {
            None
        }
    }

    pub fn update(&mut self, i: usize, j: usize, tile: TileAppearance) -> Result<(), MapError> {
        if self.is_in_bounds(i, j) {
            self.grid[i][j] = tile;
            return Ok(());
        } else {
            return Err(MapError::OutOfBounds);
        }
    }

    pub fn render(&self, commands: &mut Commands, font: Handle<Font>, bound_state: GameState) {
        for (location, tile) in self.enumerated().iter() {
            let _tile_entity = tile.render(
                &mut (commands.spawn_empty()),
                font.clone(),
                bound_state,
                Self::tile_to_world_coordinates(*location),
            );
        }
    }

    pub fn tile_to_world_coordinates(location: TileLocation) -> Vec2 {
        Vec2::new(
            (location.i * (TILE_WIDTH as i32)) as f32,
            (location.j * (TILE_HEIGHT as i32)) as f32,
        )
    }

    fn enumerated(&self) -> Vec<(TileLocation, TileAppearance)> {
        self.grid
            .iter()
            .enumerate()
            .map(|(i, row)| {
                row.iter()
                    .enumerate()
                    .map(|(j, tile)| (TileLocation::new(i as i32, j as i32), tile.clone()))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    fn is_in_bounds(&self, i: usize, j: usize) -> bool {
        if self.grid.len() == 0 {
            false
        } else if self.grid[0].len() == 0 {
            false
        } else {
            (i < self.grid.len()) && (j < self.grid[0].len())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TileAppearance {
    Ascii(AsciiTileAppearance),
    Sprite(Handle<TextureAtlas>),
}

impl TileAppearance {
    pub fn from_tile(tile: &Tile) -> Self {
        match tile.get_top_of_stack() {
            Some(object_tile) => {
                panic!("Not implemented yet.");
            }
            None => match tile.get_surface() {
                SurfaceTile::Ground => TileAppearance::Ascii(MIDDLE_DOT.into()),
                SurfaceTile::Wall => TileAppearance::Ascii('#'.into()),
                SurfaceTile::Fireplace => {
                    TileAppearance::Ascii(AsciiTileAppearance::new('X', ColorCode::Red))
                }
            },
        }
    }

    pub fn render(
        &self,
        entity_commands: &mut EntityCommands,
        font: Handle<Font>,
        bound_state: GameState,
        location: Vec2,
    ) -> Entity {
        match self {
            TileAppearance::Ascii(appearance) => entity_commands
                .insert(Text2dBundle {
                    text: Text::from_section(
                        appearance.character,
                        TextStyle {
                            font: font.clone(),
                            font_size: ASCII_TILE_FONT_SIZE,
                            color: appearance.color_code.to_color(),
                        },
                    ),
                    transform: Transform::from_translation(location.extend(0.)),
                    ..Default::default()
                })
                .insert(BoundStateComponent(bound_state))
                .id(),
            TileAppearance::Sprite(_) => panic!("Not implemented yet."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AsciiTileAppearance {
    character: char,
    color_code: ColorCode,
}

impl From<char> for AsciiTileAppearance {
    fn from(value: char) -> Self {
        Self::character(value)
    }
}

impl AsciiTileAppearance {
    pub fn new(character: char, color: ColorCode) -> Self {
        Self {
            character,
            color_code: color,
        }
    }

    pub fn character(character: char) -> Self {
        Self::new(character, ColorCode::AntiqueWhite)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorCode {
    AntiqueWhite,
    Red,
    Gray,
}

impl ColorCode {
    pub fn to_color(&self) -> Color {
        match self {
            Self::AntiqueWhite => Color::ANTIQUE_WHITE,
            Self::Red => Color::RED,
            Self::Gray => Color::GRAY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_is_traversable() {
        let map_layer = MapLayer::fill(5, 5, Tile::empty_ground());
        assert!(map_layer.is_traversable(TileLocation::zero()).unwrap());

        let map: Map = MapLayer::fill(5, 5, Tile::empty_ground()).into();
        assert!(map
            .is_traversable(MapLocation::new(0, TileLocation::zero()))
            .unwrap());
        assert!(map
            .get_layer(0)
            .unwrap()
            .is_traversable(TileLocation::zero())
            .unwrap());
        assert!(map
            .is_traversable(MapLocation::new(0, TileLocation::new(4, 4)))
            .unwrap());

        let map: Map = MapLayer::fill(5, 5, Tile::wall()).into();
        assert_eq!(
            map.is_traversable(MapLocation::new(0, TileLocation::zero()))
                .unwrap(),
            false
        );
        assert_eq!(
            map.get_layer(0)
                .unwrap()
                .is_traversable(TileLocation::zero())
                .unwrap(),
            false
        );
        assert_eq!(
            map.is_traversable(MapLocation::new(0, TileLocation::new(4, 4)))
                .unwrap(),
            false
        );
    }

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
