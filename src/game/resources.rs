use bevy::prelude::*;
use rand::rngs::ThreadRng;

use crate::game::constants::*;

use super::map::Map;

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
    Menu,
    Paused,
    GameOver,
    Victory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct LoadedFont(pub Handle<Font>);

#[derive(Debug, PartialEq, Eq, Hash, Resource)]
pub struct LoadedMap(pub Map);
