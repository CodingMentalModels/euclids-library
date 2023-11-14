use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

use super::{
    character::{ActionClockComponent, LocationComponent},
    events::Direction,
    map::MapLocation,
};
use crate::constants::*;

// Systems

// End Systems

// Components
#[derive(Component, Clone, Copy)]
pub struct EnemyComponent;

#[derive(Component, Clone)]
pub struct AIComponent {
    commands: Vec<AICommand>,
    pointer: usize,
}

impl AIComponent {
    pub fn new(commands: Vec<AICommand>) -> Self {
        if commands.len() == 0 {
            panic!("AIComponents should be non-empty.");
        }
        Self {
            commands,
            pointer: 0,
        }
    }

    pub fn next(&mut self) -> AICommand {
        self.advance_pointer();
        self.get_current_command()
    }

    pub fn get_current_command(&self) -> AICommand {
        self.commands[self.pointer].clone()
    }

    pub fn advance_pointer(&mut self) {
        self.pointer = (self.pointer + 1) % self.commands.len()
    }
}

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
            .insert(AIComponent::new(self.commands.clone()))
            .insert(ActionClockComponent(0))
            .insert(LocationComponent(location))
            .id()
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommand {
    Wait(u8),
    Move(Direction),
    MoveTowardsPlayer,
    Attack,
    Speak(String),
}

impl AICommand {
    pub fn get_ticks(&self) -> u8 {
        match self {
            Self::Wait(ticks) => *ticks,
            Self::Move(_direction) => MOVEMENT_TICKS,
            Self::MoveTowardsPlayer => MOVEMENT_TICKS,
            Self::Attack => MOVEMENT_TICKS,
            Self::Speak(_s) => AI_SPEAK_TICKS,
        }
    }
}
// End Structs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_component_cycles_correctly() {
        let mut ai = AIComponent::new(vec![AICommand::Wait(10)]);
        assert_eq!(ai.get_current_command(), AICommand::Wait(10));
        assert_eq!(ai.next(), AICommand::Wait(10));
        assert_eq!(ai.next(), AICommand::Wait(10));
        assert_eq!(ai.next(), AICommand::Wait(10));

        let mut ai = AIComponent::new(vec![
            AICommand::Wait(10),
            AICommand::Wait(5),
            AICommand::Speak("Something".to_string()),
        ]);
        assert_eq!(ai.get_current_command(), AICommand::Wait(10));
        assert_eq!(ai.next(), AICommand::Wait(5));
        assert_eq!(ai.next(), AICommand::Speak("Something".to_string()));
        assert_eq!(ai.next(), AICommand::Wait(10));
        assert_eq!(ai.next(), AICommand::Wait(5));
    }
}
