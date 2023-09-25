use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use super::constants::*;
use super::map::{SurfaceTile, Tile, TileLocation};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Resource)]
pub struct ExploringUIState {
    pub tile_grid: TileGrid,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct TileGrid {
    grid: Vec<Vec<TileAppearance>>,
}

impl TileGrid {
    pub fn new(grid: Vec<Vec<TileAppearance>>) -> Self {
        Self { grid }
    }

    pub fn get(&self, i: usize, j: usize) -> Option<TileAppearance> {
        if self.is_in_bounds(i, j) {
            Some(self.grid[i][j].clone())
        } else {
            None
        }
    }

    pub fn update(&mut self, i: usize, j: usize, tile: TileAppearance) -> Result<(), UIStateError> {
        if self.is_in_bounds(i, j) {
            self.grid[i][j] = tile;
            return Ok(());
        } else {
            return Err(UIStateError::IndexOutOfBounds);
        }
    }

    pub fn render(&self, mut entity_commands: EntityCommands, font: Handle<Font>) {
        for (location, tile) in self.enumerated().iter() {
            tile.render(
                &mut entity_commands,
                font.clone(),
                tile_to_world_coordinates(*location),
            );
        }
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
    Ascii(char),
    Sprite(Handle<TextureAtlas>),
}

impl TileAppearance {
    pub fn from_tile(tile: &Tile) -> Self {
        match tile.get_top_of_stack() {
            Some(object_tile) => {
                panic!("Not implemented yet.");
            }
            None => match tile.get_surface() {
                SurfaceTile::Ground => TileAppearance::Ascii(MIDDLE_DOT),
                SurfaceTile::Wall => TileAppearance::Ascii('#'),
            },
        }
    }

    pub fn render(
        &self,
        mut entity_commands: &mut EntityCommands,
        font: Handle<Font>,
        location: Vec2,
    ) -> Entity {
        info!("Rendering tile at {}", location);
        match self {
            TileAppearance::Ascii(character) => entity_commands
                .insert(Text2dBundle {
                    text: Text::from_section(
                        *character,
                        TextStyle {
                            font: font.clone(),
                            font_size: ASCII_TILE_FONT_SIZE,
                            color: Color::BLUE,
                        },
                    ),
                    transform: Transform::from_translation(location.extend(0.)),
                    ..Default::default()
                })
                .id(),
            TileAppearance::Sprite(_) => panic!("Not implemented yet."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIStateError {
    IndexOutOfBounds,
}

// Helper Functions
fn tile_to_world_coordinates(location: TileLocation) -> Vec2 {
    Vec2::new(
        (location.i * (TILE_WIDTH as i32)) as f32,
        (location.j * (TILE_HEIGHT as i32)) as f32,
    )
}

// End Helper Functions
