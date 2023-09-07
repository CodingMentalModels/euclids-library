use bevy::prelude::*;
use rand::rngs::ThreadRng;

use crate::game::constants::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub struct PausedState(pub GameState);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States, Resource)]
pub enum GameState {
    #[default]
    LoadingAssets,
    LoadingUI,
    Exploring,
    Menu,
    Paused,
    GameOver,
    Victory,
}
