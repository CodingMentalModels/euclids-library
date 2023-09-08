use bevy::prelude::*;

use super::{
    constants::*,
    events::MovementEvent,
    resources::GameState,
    tiles::{Tile, TileGrid},
};

pub struct ExploringPlugin;

impl Plugin for ExploringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LoadingMap), load_map_system)
            .add_systems(
                Update,
                move_player_system.run_if(in_state(GameState::Exploring)),
            );
    }
}

// Systems

pub fn load_map_system(mut commands: Commands) {
    let mut tile_grid = TileGrid::fill(
        DEFAULT_MAP_WIDTH_IN_TILES,
        DEFAULT_MAP_HEIGHT_IN_TILES,
        Tile::Ascii(MIDDLE_DOT),
    );

    commands.insert_resource(tile_grid);
    commands.insert_resource(NextState(Some(GameState::Exploring)));
}

pub fn move_player_system(
    mut movement_event_reader: EventReader<MovementEvent>,
    mut player_query: Query<(&mut LocationComponent), With<PlayerComponent>>,
) {
    let mut player_location = player_query.single_mut();

    for movement_event in movement_event_reader.iter() {
        player_location.0 = player_location.0 + movement_event.as_vector();
    }
}

// End Systems

// Components

#[derive(Component, Clone, Copy)]
pub struct PlayerComponent;

#[derive(Component, Clone, Copy)]
pub struct LocationComponent(Vec2);
// End Components

// Helper Functions

// End Helper Functions
