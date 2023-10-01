use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::constants::*;
use super::interacting::InteractingState;
use super::map::{MapLayer, SurfaceTile, Tile, TileLocation};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Resource)]
pub struct ExploringUIState {
    pub tile_grid: TileGrid,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Resource)]
pub struct InteractingUIState {
    pub tile_grid: TileGrid,
    pub interacting_state: InteractingState,
}

impl From<ExploringUIState> for InteractingUIState {
    fn from(value: ExploringUIState) -> Self {
        Self {
            tile_grid: value.tile_grid,
            interacting_state: InteractingState::default(),
        }
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

    pub fn update(&mut self, i: usize, j: usize, tile: TileAppearance) -> Result<(), UIStateError> {
        if self.is_in_bounds(i, j) {
            self.grid[i][j] = tile;
            return Ok(());
        } else {
            return Err(UIStateError::IndexOutOfBounds);
        }
    }

    pub fn render(&self, commands: &mut Commands, font: Handle<Font>) {
        for (location, tile) in self.enumerated().iter() {
            let _tile_entity = tile.render(
                &mut (commands.spawn_empty()),
                font.clone(),
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
        location: Vec2,
    ) -> Entity {
        info!("Rendering tile at {}", location);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIStateError {
    IndexOutOfBounds,
}

// Helper Functions

// End Helper Functions
