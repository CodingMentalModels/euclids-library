use std::unimplemented;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui::{Align2, Color32, Frame};

use crate::constants::*;
use crate::game::events::MenuInputEvent;
use crate::game::map::Map;
use crate::game::resources::GameState;
use crate::menu::{MenuType, MenuUIState};

// Paradigm: Vim inspired map editor.  If vim has a way to do it, then we do it that way.
// Basic Flow:
//  1. Load existing or new?
//  2. (If new) pick a name
//  3. (If new) pick a size
//  4. Use vim-like keybindings to move around, insert, copy, etc.
//
//  Everything should save on any change
//  But, within the current session, undo and redo should be possible.
//  Recording macros should be possible
//  Block mode should create rectangular blocks rather than be line-based etc.

pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MapEditorMenuEvent>()
            .add_systems(
                OnEnter(GameState::EditingMapMenu),
                initialize_map_editor_menu_system,
            )
            .add_systems(
                Update,
                render_map_editor_menu_system.run_if(in_state(GameState::EditingMapMenu)),
            );
    }
}

// Resources
#[derive(Debug, Default, PartialEq, Eq, Hash, Resource)]
pub enum MapEditorMenuUIState {
    #[default]
    NewOrLoadMenu,
    NewMapOptionsMenu,
    LoadMapMenu(Vec<String>),
}

#[derive(Debug, PartialEq, Eq, Hash, Resource)]
pub struct MapEditorEditingUIState {
    filename: String,
    map: Map,
    current_layer: usize,
    mode: EditingMode,
}

impl MapEditorEditingUIState {
    pub fn new(filename: String, map: Map, current_layer: usize, mode: EditingMode) -> Self {
        Self {
            filename,
            map,
            current_layer,
            mode,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum EditingMode {
    #[default]
    Normal,
    Insert,
    Block,
}

// End Resources

// Events
#[derive(Debug, Clone, Hash, PartialEq, Eq, Event)]
pub enum MapEditorMenuEvent {
    NewMap(String),
    LoadMap(String),
}

// End Events

// Systems
fn initialize_map_editor_menu_system(mut commands: Commands) {
    commands.insert_resource(MapEditorMenuUIState::default());
}

fn render_map_editor_menu_system(
    mut contexts: EguiContexts,
    mut input_event_reader: EventReader<MenuInputEvent>,
    mut map_editor_menu_event_writer: EventWriter<MapEditorMenuEvent>,
    mut ui_state: ResMut<MapEditorMenuUIState>,
) {
    match &mut *ui_state {
        MapEditorMenuUIState::NewOrLoadMenu => {
            let menu = MenuUIState::new(MenuType::SelectFinite(vec![
                "New Map".to_string(),
                "Load Map".to_string(),
            ]));
            let response = menu.render(&mut contexts, &mut input_event_reader);
            // TODO Load this from folder
            let maps = vec![];
            match response {
                Some("New Map") => *ui_state = MapEditorMenuUIState::NewMapOptionsMenu,
                Some("Load Map") => *ui_state = MapEditorMenuUIState::LoadMapMenu(maps),
                Some(_) => panic!("There are only two options."),
                None => {}
            };
        }
        MapEditorMenuUIState::NewMapOptionsMenu => {
            let menu = MenuUIState::new(MenuType::Info(vec!["New Map Options Menu".to_string()]));
            let _response = menu.render(&mut contexts, &mut input_event_reader);
        }
        MapEditorMenuUIState::LoadMapMenu(maps) => {
            let menu = MenuUIState::new(MenuType::Info(vec!["Load Map Menu".to_string()]));
            let _response = menu.render(&mut contexts, &mut input_event_reader);
        }
    }
}

// End Systems
