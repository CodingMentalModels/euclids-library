use bevy::prelude::*;

use super::resources::GameState;

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
    spawn_player(&mut commands, 0, Vec2::ZERO);

    commands.insert_resource(NextState(Some(GameState::LoadingMap)));
}

// End Systems

// Components
#[derive(Component, Clone, Copy)]
pub struct PlayerComponent;

#[derive(Component, Clone, Copy)]
pub struct LocationComponent {
    pub map_layer: usize,
    pub location: Vec2,
}

impl LocationComponent {
    pub fn translate(&mut self, amount: Vec2) {
        self.location += amount;
    }
}

// End Components

// Helper Functions

fn spawn_player(commands: &mut Commands, map_layer: usize, location: Vec2) -> Entity {
    commands
        .spawn_empty()
        .insert(LocationComponent {
            map_layer,
            location,
        })
        .insert(PlayerComponent)
        .id()
}

// End Helper Functions
