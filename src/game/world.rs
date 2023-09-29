use bevy::prelude::*;

use super::{
    map::{MapLocation, TileLocation},
    player::Player,
    resources::{GameState, LoadedFont, NPCSpecs},
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InitializingWorld),
            initialize_player_system,
        )
        .add_systems(
            OnEnter(GameState::InitializingWorld),
            initialize_npcs_system,
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

fn initialize_npcs_system(mut commands: Commands, npc_specs: Res<NPCSpecs>) {
    for (name, npc) in npc_specs.0.as_vec().into_iter() {
        npc.spawn(&mut commands.spawn_empty());
    }
}

// End Systems

// Components

// End Components

// Helper Functions

// End Helper Functions

// Helper Structs

// End Helper Structs
