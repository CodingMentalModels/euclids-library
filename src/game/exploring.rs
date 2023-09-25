use bevy::prelude::*;

use super::{
    constants::*,
    events::{CameraMovementEvent, MovementEvent},
    map::{MapLayer, SurfaceTile, Tile},
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
            )
            .add_systems(
                Update,
                move_camera_system.run_if(in_state(GameState::Exploring)),
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
    map_layer
        .update(5, 5, Tile::empty(SurfaceTile::Fireplace))
        .expect("5, 5 exists because we're setting it up that way.");

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

pub fn move_camera_system(
    mut camera_movement_event_reader: EventReader<CameraMovementEvent>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let mut transform = query.single_mut();
    for event in camera_movement_event_reader.iter() {
        transform.translation += event.0.as_vector().extend(0.) * CAMERA_MOVE_SPEED;
        info!("New transform: {:?}", transform.translation);
    }
}

// End Systems

// Components

// End Components

// Helper Functions

// End Helper Functions
