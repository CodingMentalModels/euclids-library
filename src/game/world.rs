use bevy::prelude::*;

use super::{
    map::{MapLocation, TileLocation},
    player::Player,
    resources::GameState,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InitializingWorld),
            initialize_player_system,
        );
    }
}

// Systems

fn initialize_player_system(mut commands: Commands) {
    let player = Player::new(
        "Player".to_string(),
        MapLocation::new(0, TileLocation::zero()),
    );
    player.spawn(&mut commands);

    commands.insert_resource(NextState(Some(GameState::LoadingMap)));
}

// End Systems

// Components

// End Components

// Helper Functions

// End Helper Functions

// Helper Structs

// End Helper Structs
