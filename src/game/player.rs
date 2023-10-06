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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Damage {
    Piercing(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPartTreeNode {
    body_part: BodyPart,
    children: Box<Vec<BodyPartTreeNode>>,
}

impl BodyPartTreeNode {
    pub fn new(body_part: BodyPart, children: Vec<BodyPartTreeNode>) -> Self {
        Self {
            body_part,
            children: Box::new(children),
        }
    }

    pub fn leaf(body_part: BodyPart) -> Self {
        Self::new(body_part, Vec::new())
    }

    pub fn new_humanoid() -> Self {
        let mut body = Self::leaf(BodyPartType::Body.into());
        body.add(Self::new_humanoid_head());

        body.add(Self::new_leg_and_foot());
        body.add(Self::new_leg_and_foot());

        body.add(Self::new_arm_and_hand());
        body.add(Self::new_arm_and_hand());

        body
    }

    pub fn new_humanoid_head() -> Self {
        let mut head = Self::leaf(BodyPartType::Head.into());

        head.add_leaf(BodyPartType::Eye.into());
        head.add_leaf(BodyPartType::Eye.into());

        head.add_leaf(BodyPartType::Ear.into());
        head.add_leaf(BodyPartType::Ear.into());

        head.add_leaf(BodyPartType::Mouth.into());

        head
    }

    pub fn new_leg_and_foot() -> Self {
        let mut leg = Self::leaf(BodyPartType::Leg.into());
        leg.add_leaf(BodyPartType::Foot.into());
        leg
    }

    pub fn new_arm_and_hand() -> Self {
        let mut arm = Self::leaf(BodyPartType::Arm.into());
        arm.add_leaf(BodyPartType::Hand.into());
        arm
    }

    pub fn add(&mut self, body_part: BodyPartTreeNode) {
        self.children.push(body_part);
    }

    pub fn add_leaf(&mut self, body_part: BodyPart) {
        self.add(Self::leaf(body_part))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    body_part_type: BodyPartType,
    statuses: Vec<BodyPartStatus>,
}

impl From<BodyPartType> for BodyPart {
    fn from(value: BodyPartType) -> Self {
        Self::new_empty(value)
    }
}

impl BodyPart {
    pub fn new(body_part_type: BodyPartType, statuses: Vec<BodyPartStatus>) -> Self {
        Self {
            body_part_type,
            statuses,
        }
    }

    pub fn new_empty(body_part_type: BodyPartType) -> Self {
        Self::new(body_part_type, Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyPartType {
    Body,
    Head,
    Brain,
    Eye,
    Ear,
    Mouth,
    Arm,
    Hand,
    Leg,
    Foot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyPartStatus {
    Bleeding(u32),
    Infected,
    Cancerous,
    Nonfunctional,
}

// End Structs
