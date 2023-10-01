use bevy::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use super::map::MapLocation;
use super::map::TileLocation;

// Components
#[derive(Component, Clone, Copy)]
pub struct PlayerComponent;

#[derive(Component, Clone, Copy, PartialEq)]
pub struct LocationComponent(pub MapLocation);

impl LocationComponent {
    pub fn translate(&mut self, amount: TileLocation) {
        self.0.translate(amount);
    }

    pub fn translated(&self, amount: TileLocation) -> Self {
        let mut to_return = self.clone();
        to_return.translate(amount);
        to_return
    }
}

// End Components

// Structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    name: String,
    location: MapLocation,
}

impl Player {
    pub fn new(name: String, location: MapLocation) -> Self {
        Self { name, location }
    }

    pub fn spawn(&self, commands: &mut Commands) -> Entity {
        commands
            .spawn_empty()
            .insert(LocationComponent(self.location))
            .insert(PlayerComponent)
            .id()
    }
}

// End Structs
