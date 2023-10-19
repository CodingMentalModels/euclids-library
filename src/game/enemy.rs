use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

use super::{
    character::{ActionClockComponent, LocationComponent},
    constants::*,
    map::MapLocation,
};

// Systems

// End Systems

// Components
#[derive(Component, Clone, Copy)]
pub struct EnemyComponent;

#[derive(Component, Clone)]
pub struct EnemyAIComponent(Vec<AICommand>);

// End Components

// Structs
#[derive(Clone, Serialize, Deserialize)]
pub struct Enemy {
    commands: Vec<AICommand>,
}

impl Enemy {
    pub fn new(commands: Vec<AICommand>) -> Self {
        Self { commands }
    }

    pub fn spawn(&self, entity_commands: &mut EntityCommands, location: MapLocation) -> Entity {
        entity_commands
            .insert(EnemyComponent)
            .insert(EnemyAIComponent(self.commands.clone()))
            .insert(ActionClockComponent(0))
            .insert(LocationComponent(location))
            .id()
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub enum AICommand {
    Wait(u8),
    Move(Direction),
    Speak(String),
}

impl AICommand {
    pub fn get_ticks(&self) -> u8 {
        match self {
            Self::Wait(ticks) => *ticks,
            Self::Move(_direction) => MOVEMENT_TICKS,
            Self::Speak(_s) => AI_SPEAK_TICKS,
        }
    }
}
// End Structs
