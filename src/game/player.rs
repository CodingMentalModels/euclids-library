use std::collections::HashMap;
use std::collections::HashSet;

use bevy::prelude::*;

use rand::rngs::ThreadRng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

use super::character::BodyComponent;
use super::character::BodyPartTreeNode;
use super::character::LocationComponent;
use super::constants::*;
use super::map::MapLocation;
use super::map::TileLocation;

// Components
#[derive(Component, Clone, Copy)]
pub struct PlayerComponent;

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
            .insert(PlayerComponent)
            .insert(LocationComponent(self.location))
            .insert(BodyComponent(BodyPartTreeNode::new_humanoid()))
            .id()
    }
}

// End Structs
