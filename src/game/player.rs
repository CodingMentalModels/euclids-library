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
pub struct BodyPartTreeNode {
    body_part: BodyPart,
    children: Box<Vec<BodyPartTreeNode>>,
}

impl From<BodyPart> for BodyPartTreeNode {
    fn from(value: BodyPart) -> Self {
        Self::leaf(value)
    }
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
        body.add(Self::new_head());

        body.add(Self::new_leg_and_foot());
        body.add(Self::new_leg_and_foot());

        body.add(Self::new_arm_and_hand());
        body.add(Self::new_arm_and_hand());

        body
    }

    pub fn new_head() -> Self {
        let mut head = Self::leaf(BodyPartType::Head.into());

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

    pub fn has_status_effect(&self, status_effect: BodyPartStatusEffect) -> PartialBool {
        self.body_part.has_status_effect(status_effect)
    }

    pub fn has_status_effect_recursive(&self, status_effect: BodyPartStatusEffect) -> bool {
        let own_has_status_effect = self.has_status_effect(status_effect);
        let children_has_status_effect = self
            .children
            .iter()
            .all(|child| child.has_status_effect_recursive(status_effect));
        return (own_has_status_effect != PartialBool::False) && children_has_status_effect;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    body_part_type: BodyPartType,
    state: BodyPartState,
    statuses: Vec<BodyPartStatusEffect>,
}

impl From<BodyPartType> for BodyPart {
    fn from(value: BodyPartType) -> Self {
        Self::new_empty(value)
    }
}

impl BodyPart {
    pub fn new(
        body_part_type: BodyPartType,
        state: BodyPartState,
        statuses: Vec<BodyPartStatusEffect>,
    ) -> Self {
        Self {
            body_part_type,
            state,
            statuses,
        }
    }

    pub fn new_empty(body_part_type: BodyPartType) -> Self {
        Self::new(body_part_type, BodyPartState::Okay, Vec::new())
    }

    pub fn new_with_status_effects(
        body_part_type: BodyPartType,
        status_effects: Vec<BodyPartStatusEffect>,
    ) -> Self {
        Self::new(body_part_type, BodyPartState::Okay, status_effects)
    }

    pub fn has_status_effect(&self, status_effect: BodyPartStatusEffect) -> PartialBool {
        if self
            .body_part_type
            .is_status_effect_applicable(status_effect)
        {
            self.statuses.contains(&status_effect).into()
        } else {
            PartialBool::NotApplicable
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyPartType {
    Body,
    Head,
    Arm,
    Hand,
    Leg,
    Foot,
}

impl BodyPartType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Body,
            Self::Head,
            Self::Arm,
            Self::Hand,
            Self::Leg,
            Self::Foot,
        ]
    }

    pub fn is_status_effect_applicable(&self, status_effect: BodyPartStatusEffect) -> bool {
        status_effect.get_applicable_bodyparts().contains(&self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyPartState {
    Okay,
    Nonfunctional,
    Destroyed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyPartStatusEffect {
    Blind,
    Deaf,
    Bleeding,
    Infected,
    Cancerous,
}

impl BodyPartStatusEffect {
    pub fn get_applicable_bodyparts(&self) -> Vec<BodyPartType> {
        match self {
            Self::Blind | Self::Deaf => vec![BodyPartType::Head],
            _ => BodyPartType::all(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartialBool {
    True,
    False,
    NotApplicable,
}

impl From<bool> for PartialBool {
    fn from(value: bool) -> Self {
        if value {
            PartialBool::True
        } else {
            PartialBool::False
        }
    }
}

// End Structs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_part_has_status_effect_recursive() {
        let humanoid = BodyPartTreeNode::new_humanoid();
        assert_eq!(
            humanoid.has_status_effect_recursive(BodyPartStatusEffect::Blind),
            false
        );

        let head: BodyPartTreeNode = BodyPart::new_with_status_effects(
            BodyPartType::Head,
            vec![BodyPartStatusEffect::Blind],
        )
        .into();

        assert!(head.has_status_effect_recursive(BodyPartStatusEffect::Blind));
        assert_eq!(
            head.has_status_effect_recursive(BodyPartStatusEffect::Infected),
            false
        );

        let mut body = BodyPartTreeNode::new(BodyPartType::Body.into(), vec![head]);
        assert!(body.has_status_effect_recursive(BodyPartStatusEffect::Blind));
        assert_eq!(
            body.has_status_effect_recursive(BodyPartStatusEffect::Infected),
            false
        );

        body.add_leaf(BodyPartType::Arm.into());
        assert!(body.has_status_effect_recursive(BodyPartStatusEffect::Blind));
        assert_eq!(
            body.has_status_effect_recursive(BodyPartStatusEffect::Infected),
            false
        );

        body.add_leaf(BodyPartType::Head.into());
        assert_eq!(
            body.has_status_effect_recursive(BodyPartStatusEffect::Blind),
            false
        );
        assert_eq!(
            body.has_status_effect_recursive(BodyPartStatusEffect::Infected),
            false
        );
    }
}
