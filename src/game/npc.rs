use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

use super::{
    dialog::{Dialog, DialogComponent},
    map::MapLocation,
    player::LocationComponent,
};

// Components

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

    pub fn spawn(&self, entity_commands: &mut EntityCommands, font: Handle<Font>) -> Entity {
        entity_commands
            .insert(LocationComponent(self.location))
            .insert(DialogComponent(self.dialog.clone()))
            .id()
    }
}
