use bevy::prelude::*;

use super::{
    constants::*,
    tiles::{Tile, TileGrid},
};

pub struct ExploringPlugin;

impl Plugin for ExploringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Exploring), load_map_system);
    }
}

// Systems

pub fn load_map_system(mut commands: Commands) {
    let tile_grid = TileGrid::fill(
        DEFAULT_MAP_WIDTH,
        DEFAULT_MAP_HEIGHT,
        Tile::Ascii(MIDDLE_DOT),
    );

    commands.insert_resource(tile_grid);
}

// End Systems

// Helper Functions

// End Helper Functions
