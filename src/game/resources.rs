use std::{collections::BTreeMap, sync::RwLock};

use bevy::prelude::*;
use rand::rngs::StdRng;

use crate::game::constants::*;

use super::{map::Map, npc::NPC, specs::SpecLookup};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub struct PausedState(pub GameState);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States, Resource)]
pub enum GameState {
    #[default]
    LoadingAssets,
    LoadingUI,
    InitializingWorld,
    LoadingMap,
    Exploring,
    NonPlayerTurns,
    Interacting,
    Menu,
    Paused,
    GameOver,
    Victory,
}

#[derive(Debug, Resource)]
pub struct RngResource(pub RwLock<StdRng>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct LoadedFont(pub Handle<Font>);

#[derive(Debug, PartialEq, Eq, Hash, Resource)]
pub struct LoadedMap(pub Map);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct MaterialCache(pub BTreeMap<u32, Handle<ColorMaterial>>);

impl MaterialCache {
    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(&mut self, color: &Color, handle: Handle<ColorMaterial>) {
        self.0.insert(color.as_rgba_u32(), handle);
    }

    pub fn get(&self, color: &Color) -> Option<Handle<ColorMaterial>> {
        self.0.get(&color.as_rgba_u32()).cloned()
    }
}
// Specs

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct NPCSpecs(pub SpecLookup<NPC>);

impl NPCSpecs {
    pub fn from_vec(npcs: Vec<NPC>) -> Self {
        Self(SpecLookup::from_vec(npcs, |npc| npc.name.clone()))
    }
}

// End Specs
