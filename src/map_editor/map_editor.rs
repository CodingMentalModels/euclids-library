use bevy::prelude::*;

// Basic Flow:
//  1. Load existing or new?
//  2. (If new) pick a name
//  3. (If new) pick a size
//  4. Use vim-like keybindings to move around, insert, copy, etc.
//
//  Everything should save on any change
//  Recording macros should be possible
//  Block mode should create rectangular blocks rather than be line-based etc.

pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(GameState::LoadingMap), load_map_system)
        //     .add_systems(Update, movement_system.run_if(generalized_exploring()))
    }
}
