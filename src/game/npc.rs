use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

use super::{
    dialog::Dialog,
    interacting::{Interactable, InteractableComponent},
    map::MapLocation,
    player::{BodyComponent, BodyPartTreeNode, LocationComponent},
};

// Components

#[derive(Component, Clone, Copy)]
pub struct NPCComponent;

// End Components

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NPC {
    pub name: String,
    pub location: MapLocation,
    pub dialog: Dialog,
}

impl NPC {
    pub fn new(name: String, location: MapLocation, dialog: Dialog) -> Self {
        Self {
            name,
            location,
            dialog,
        }
    }

    pub fn spawn(&self, entity_commands: &mut EntityCommands) -> Entity {
        entity_commands
            .insert(NPCComponent)
            .insert(LocationComponent(self.location))
            .insert(BodyComponent(BodyPartTreeNode::new_humanoid()))
            .insert(InteractableComponent(Interactable::Dialog(
                self.dialog.clone(),
            )))
            .id()
    }
}
