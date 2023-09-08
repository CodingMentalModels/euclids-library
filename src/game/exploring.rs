use bevy::prelude::*;

use super::{
    constants::*,
    resources::GameState,
    tiles::{Tile, TileGrid},
};

pub struct ExploringPlugin;

impl Plugin for ExploringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LoadingMap), load_map_system);
    }
}

// Systems

pub fn load_map_system(mut commands: Commands) {
    let tile_grid = TileGrid::fill(
        DEFAULT_MAP_WIDTH_IN_TILES,
        DEFAULT_MAP_HEIGHT_IN_TILES,
        Tile::Ascii(MIDDLE_DOT),
    );

    commands.insert_resource(tile_grid);
    commands.insert_resource(NextState(Some(GameState::Exploring)));
}

// End Systems

// Helper Functions

// End Helper Functions
