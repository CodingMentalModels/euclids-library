use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::constants::*;
use super::map::{MapLocation, TileLocation};
use super::resources::RngResource;

// Components

#[derive(Component, Clone)]
pub struct BodyComponent(pub BodyPartTreeNode);

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

#[derive(Component, Clone, Copy, PartialEq)]
pub struct ActionClockComponent(pub u8);

impl ActionClockComponent {
    pub fn tick(&mut self, amount: u8) -> bool {
        self.0 = self.0.saturating_sub(amount);
        self.finished()
    }

    pub fn finished(&self) -> bool {
        self.0 == 0
    }
}
// End Components

// Structs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Damage {
    state_transition_probabilities: HashMap<(BodyPartState, BodyPartState), Probability>,
    status_effect_probabilities: std::collections::HashMap<BodyPartStatusEffect, Probability>,
}

impl Damage {
    pub fn new(
        state_transition_probabilities: HashMap<(BodyPartState, BodyPartState), Probability>,
        status_effect_probabilities: HashMap<BodyPartStatusEffect, Probability>,
    ) -> Result<Self, ProbabilityError> {
        // Check that totals by inital don't overflow
        let _totals_by_initial: HashMap<BodyPartState, Probability> =
            state_transition_probabilities.iter().fold(
                Ok(HashMap::new()),
                |acc, ((initial_state, _final_state), p)| {
                    acc.and_then(|mut a| {
                        match a.get(initial_state) {
                            None => {
                                a.insert(*initial_state, *p);
                            }
                            Some(previous_p) => {
                                a.insert(*initial_state, previous_p.add(p)?);
                            }
                        };

                        Ok(a)
                    })
                },
            )?;

        Ok(Self {
            state_transition_probabilities,
            status_effect_probabilities,
        })
    }

    pub fn get_state_transitions_from(
        &self,
        initial_state: BodyPartState,
    ) -> Result<HashMap<BodyPartState, Probability>, ProbabilityError> {
        let final_states = self
            .state_transition_probabilities
            .iter()
            .filter(|((i, f), _p)| (initial_state == *i) && (*f != initial_state))
            .map(|((_i, f), _p)| f)
            .cloned()
            .collect::<Vec<_>>();
        let mut state_transitions = final_states
            .iter()
            .map(|f| {
                (
                    *f,
                    *self
                        .state_transition_probabilities
                        .get(&(initial_state, *f))
                        .expect("This key was generated from the hashmap itself."),
                )
            })
            .collect::<HashMap<_, _>>();

        let remaining_probabilities =
            Probability::new(100 - state_transitions.iter().fold(0, |acc, elt| acc + (elt.1).0))?;
        state_transitions.insert(initial_state, remaining_probabilities);
        Ok(state_transitions)
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

    pub fn take_damage_recursive(&mut self, rng: &mut RngResource, damage: Damage) {
        let own_size = self.get_size() as f32;
        let children_size = self.get_total_children_size() as f32;
        let p_own_damage = own_size / (own_size + children_size);
        let self_takes_damage = Probability::from_f32(p_own_damage)
            .expect("p_own_damage is guaranteed to be a valid probability")
            .roll(rng);
        if self_takes_damage {
            self.take_damage(rng, damage);
        } else {
            self.take_damage_child(rng, damage);
        }
    }

    pub fn take_damage_child(&mut self, rng: &mut RngResource, damage: Damage) {
        let children_size = self.get_total_children_size() as f32;
        let choice: usize = Probability::choose(
            rng,
            self.children
                .iter()
                .enumerate()
                .map(|(child_idx, child)| {
                    (
                        child_idx,
                        Probability::from_f32((child.get_size() as f32) / children_size)
                            .expect("Given the computation, this has to be ok."),
                    )
                })
                .collect(),
        )
        .expect("Given the computation, this has to be ok.");
        self.children[choice].take_damage_recursive(rng, damage);
    }

    pub fn take_damage(&mut self, rng: &mut RngResource, damage: Damage) {
        let state_transitions = damage
            .get_state_transitions_from(self.get_state())
            .expect("State transitions are checked when Damage is instantiated.");

        self.body_part.state = Probability::choose(rng, state_transitions)
            .expect("State transitions are checked when Damage is instantiated.");

        self.body_part.statuses.insert(
            Probability::choose(rng, damage.status_effect_probabilities)
                .expect("State transitions are checked when Damage is instantiated."),
        );
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

    pub fn get_size(&self) -> u8 {
        self.body_part.get_size()
    }

    pub fn get_total_children_size(&self) -> u8 {
        self.children
            .iter()
            .map(|child| child.get_size_recursive())
            .sum()
    }

    pub fn get_size_recursive(&self) -> u8 {
        if self.has_children() {
            self.get_size() + self.get_total_children_size()
        } else {
            self.get_size()
        }
    }

    pub fn has_children(&self) -> bool {
        self.children.len() != 0
    }

    pub fn get_state(&self) -> BodyPartState {
        self.body_part.state
    }

    pub fn get_menu_text(&self) -> Vec<String> {
        // TODO Implement in terms of LayoutJob in order to get multiple colors, etc.
        let mut own_text = vec![self.body_part.get_menu_text()];
        if self.has_children() {
            let mut children_text = self
                .children
                .iter()
                .map(|child| {
                    child
                        .get_menu_text()
                        .into_iter()
                        .map(|line| format!("{}{}", MENU_INDENTATION, line))
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();
            own_text.append(&mut children_text);
        }
        own_text
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    body_part_type: BodyPartType,
    state: BodyPartState,
    statuses: HashSet<BodyPartStatusEffect>,
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
        statuses: HashSet<BodyPartStatusEffect>,
    ) -> Self {
        Self {
            body_part_type,
            state,
            statuses,
        }
    }

    pub fn new_empty(body_part_type: BodyPartType) -> Self {
        Self::new(body_part_type, BodyPartState::Okay, HashSet::new())
    }

    pub fn new_with_status_effects(
        body_part_type: BodyPartType,
        status_effects: HashSet<BodyPartStatusEffect>,
    ) -> Self {
        Self::new(body_part_type, BodyPartState::Okay, status_effects)
    }

    pub fn get_size(&self) -> u8 {
        self.body_part_type.get_size()
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

    pub fn get_menu_text(&self) -> String {
        let head = format!(
            "{} {} {}",
            self.body_part_type.to_string(),
            EM_DASH,
            self.state.to_string(),
        );
        if self.has_any_status_effect() {
            return format!(
                "{} {} {}",
                head,
                EM_DASH,
                BodyPartStatusEffect::pretty_print_set(&self.statuses.clone())
            );
        } else {
            return head;
        }
    }

    pub fn has_any_status_effect(&self) -> bool {
        self.statuses.len() != 0
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

    pub fn to_string(&self) -> String {
        match self {
            Self::Body => "Body",
            Self::Head => "Head",
            Self::Arm => "Arm",
            Self::Hand => "Hand",
            Self::Leg => "Leg",
            Self::Foot => "Foot",
        }
        .to_string()
    }

    pub fn get_size(&self) -> u8 {
        match self {
            Self::Body => DEFAULT_BODY_SIZE,
            Self::Head => DEFAULT_HEAD_SIZE,
            Self::Arm => DEFAULT_ARM_SIZE,
            Self::Leg => DEFAULT_LEG_SIZE,
            Self::Hand => DEFAULT_HAND_SIZE,
            Self::Foot => DEFAULT_FOOT_SIZE,
        }
    }

    pub fn is_status_effect_applicable(&self, status_effect: BodyPartStatusEffect) -> bool {
        status_effect.get_applicable_bodyparts().contains(&self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyPartState {
    Okay,
    Nonfunctional,
    Destroyed,
}

impl BodyPartState {
    pub fn to_string(&self) -> String {
        match self {
            Self::Okay => "Okay",
            Self::Nonfunctional => "Nonfunctional",
            Self::Destroyed => "Destroyed",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    pub fn pretty_print_set(v: &HashSet<Self>) -> String {
        // TODO Handle overflow
        let mut to_return = v.iter().map(|e| e.to_string()).collect::<Vec<_>>();
        to_return.sort();
        to_return.join(", ")
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Blind => "Blind",
            Self::Deaf => "Deaf",
            Self::Bleeding => "Bleeding",
            Self::Infected => "Infected",
            Self::Cancerous => "Cancerous",
        }
        .to_string()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Probability(u8);

impl Probability {
    pub fn new(p: u8) -> Result<Self, ProbabilityError> {
        if p > 100 {
            return Err(ProbabilityError::OutOfBounds);
        }
        Ok(Self(p))
    }

    pub fn one() -> Self {
        Self(100)
    }

    pub fn from_f32(p: f32) -> Result<Self, ProbabilityError> {
        if p < 0. || p > 1. {
            return Err(ProbabilityError::OutOfBounds);
        }
        Ok(Self((p * 100.) as u8))
    }

    pub fn add(&self, rhs: &Probability) -> Result<Self, ProbabilityError> {
        Self::new(self.0 + rhs.0)
    }

    pub fn choose<T: Clone>(
        rng: &mut RngResource,
        choices: HashMap<T, Probability>,
    ) -> Result<T, ProbabilityError> {
        if choices.len() == 0 {
            panic!("choose() requires choices to contain at least one value.");
        }

        let total_prob: u8 = choices.values().map(|p| p.0).sum();
        let _ = Probability::new(total_prob)?;
        // Ensure that total probability is still a probability.

        let mut raw_rng = rng
            .0
            .write()
            .expect("If a thread somewhere panicked, we should panic.");
        let mut rand_choice = raw_rng.gen_range(0..=total_prob);

        for (item, prob) in choices.iter() {
            if rand_choice <= prob.0 {
                return Ok(item.clone());
            }
            rand_choice -= prob.0;
        }

        panic!("Unreachable -- the sum is calculated within the function.");
    }

    pub fn roll(&self, rng: &mut RngResource) -> bool {
        let mut raw_rng = rng
            .0
            .write()
            .expect("If a thread somewhere panicked, then we should panic.");
        let roll: f32 = raw_rng.gen();
        let roll_u8 = (roll * 100.) as u8;
        roll_u8 < self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbabilityError {
    OutOfBounds,
}

// End Structs

// Helper Functions

// End Helper Functions

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
            vec![BodyPartStatusEffect::Blind].into_iter().collect(),
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

    #[test]
    fn test_action_clock_ticks() {
        let action_clock = ActionClockComponent(0);
        assert!(action_clock.finished());

        let mut action_clock = ActionClockComponent(10);
        assert!(!action_clock.finished());
        assert_eq!(action_clock.0, 10);

        action_clock.tick(5);
        assert!(!action_clock.finished());

        action_clock.tick(0);
        assert!(!action_clock.finished());

        action_clock.tick(4);
        assert!(!action_clock.finished());
        assert_eq!(action_clock.0, 1);

        action_clock.tick(1);
        assert!(action_clock.finished());

        action_clock.tick(100);
        assert!(action_clock.finished());

        assert_eq!(action_clock.0, 0);
    }
}
