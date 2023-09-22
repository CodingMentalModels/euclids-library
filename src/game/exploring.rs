use bevy::prelude::*;

use super::{
    constants::*,
    events::MovementEvent,
    map::{MapLayer, Tile},
    player::{LocationComponent, PlayerComponent},
    resources::{GameState, LoadedMap},
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
    let mut map_layer = MapLayer::fill(
        DEFAULT_MAP_WIDTH_IN_TILES,
        DEFAULT_MAP_HEIGHT_IN_TILES,
        Tile::empty_ground(),
    );
    map_layer.update_edges(&Tile::wall());

    commands.insert_resource(LoadedMap(map_layer.into()));
    commands.insert_resource(NextState(Some(GameState::Exploring)));
}

pub fn move_player_system(
    mut movement_event_reader: EventReader<MovementEvent>,
    mut player_query: Query<&mut LocationComponent, With<PlayerComponent>>,
) {
    let mut player_location = player_query.single_mut();

    for movement_event in movement_event_reader.iter() {
        player_location.translate(movement_event.0.as_tile_location());
    }
}

// End Systems

// Components

// End Components

// Helper Functions

// End Helper Functions
