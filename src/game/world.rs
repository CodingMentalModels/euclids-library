use bevy::prelude::*;

use super::{
    enemy::{AICommand, Enemy},
    events::Direction,
    map::{MapLocation, TileLocation},
    player::Player,
    resources::{GameState, NPCSpecs},
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InitializingWorld),
            initialize_world_system,
        )
        .add_systems(
            OnEnter(GameState::InitializingWorld),
            initialize_player_system,
        )
        .add_systems(
            OnEnter(GameState::InitializingWorld),
            initialize_npcs_system,
        )
        .add_systems(
            OnEnter(GameState::InitializingWorld),
            initialize_enemies_system,
        );
    }
}

// Resources
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Resource)]
pub struct TimeElapsed(pub u64);

impl TimeElapsed {
    pub fn increment(&mut self, amount: u64) {
        self.0 = self.0 + amount;
    }
}

// End Resources

// Systems

fn initialize_world_system(mut commands: Commands) {
    commands.insert_resource(TimeElapsed::default());
}

fn initialize_player_system(mut commands: Commands) {
    let player = Player::new(
        "Player".to_string(),
        MapLocation::new(0, TileLocation::new(1, 1)),
    );
    player.spawn(&mut commands);

    commands.insert_resource(NextState(Some(GameState::LoadingMap)));
}

fn initialize_npcs_system(mut commands: Commands, npc_specs: Res<NPCSpecs>) {
    for (_name, npc) in npc_specs.0.as_vec().into_iter() {
        npc.spawn(&mut commands.spawn_empty());
    }
}

fn initialize_enemies_system(mut commands: Commands) {
    let enemies = vec![Enemy::new(vec![
        AICommand::Speak("Fear me!".to_string()),
        AICommand::Wait(30),
        AICommand::Move(Direction::UpRight),
    ])];

    // TODO Don't spawn all the enemies in the same place
    for enemy in enemies {
        enemy.spawn(
            &mut commands.spawn_empty(),
            MapLocation::new(0, TileLocation::new(9, 9)).into(),
        );
    }
}

// End Systems

// Components

// End Components

// Helper Functions

// End Helper Functions

// Helper Structs

// End Helper Structs
