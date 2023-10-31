use std::unimplemented;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui::{Align2, Color32, Frame};
use sanitize_filename::is_sanitized;

use crate::constants::*;
use crate::game::events::MenuInputEvent;
use crate::game::map::{Map, MapLayer, Tile};
use crate::game::resources::GameState;
use crate::menu::{MenuToShow, MenuType, MenuUIState};
use crate::ui::ToastMessageEvent;

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
    NewMapNameMenu,
    NewMapSizeMenu(String),
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
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut input_event_reader: EventReader<MenuInputEvent>,
    mut map_editor_menu_event_writer: EventWriter<MapEditorMenuEvent>,
    mut toast_message_event_writer: EventWriter<ToastMessageEvent>,
    mut ui_state: ResMut<MapEditorMenuUIState>,
    mut menu_to_show: ResMut<MenuToShow>,
) {
    match &mut *ui_state {
        MapEditorMenuUIState::NewOrLoadMenu => {
            let mut menu = MenuUIState::new(MenuType::SelectFinite(vec![
                "New Map".to_string(),
                "Load Map".to_string(),
            ]));
            let response = menu.render(&mut contexts, &mut input_event_reader);
            // TODO Load this from folder
            let maps = vec![];
            match response.as_deref() {
                Some("New Map") => *ui_state = MapEditorMenuUIState::NewMapNameMenu,
                Some("Load Map") => *ui_state = MapEditorMenuUIState::LoadMapMenu(maps),
                Some(_) => panic!("There are only two options."),
                None => {}
            };
        }
        MapEditorMenuUIState::NewMapNameMenu => {
            let mut menu = MenuUIState::new(MenuType::TextInput("Map Name:".to_string()));
            let response = menu.render(&mut contexts, &mut input_event_reader);
            match response {
                Some(map_name) => match sanitize_map_name(&map_name) {
                    Ok(name) => {
                        *ui_state = MapEditorMenuUIState::NewMapSizeMenu(name);
                    }
                    Err(e) => toast_message_event_writer.send(ToastMessageEvent(e.to_string())),
                },
                None => {}
            }
        }
        MapEditorMenuUIState::NewMapSizeMenu(map_name) => {
            let mut menu = MenuUIState::new(MenuType::TextInput("Map Size:".to_string()));
            let response = menu.render(&mut contexts, &mut input_event_reader);
            match response {
                Some(map_size) => match map_size.parse() {
                    Ok(size) => {
                        let filename = format!("{}.json", map_name);
                        let map = MapLayer::fill(size, size, Tile::empty_ground()).into();
                        let current_layer = 0;
                        let ui_state = MapEditorEditingUIState::new(
                            filename,
                            map,
                            current_layer,
                            EditingMode::Normal,
                        );
                        commands.insert_resource(ui_state);
                        commands.insert_resource(NextState(Some(GameState::EditingMap)));
                    }
                    Err(e) => {
                        toast_message_event_writer.send(ToastMessageEvent(e.to_string()));
                    }
                },
                None => {}
            }
        }
        MapEditorMenuUIState::LoadMapMenu(maps) => {
            let mut menu = MenuUIState::new(MenuType::Info(vec!["Load Map Menu".to_string()]));
            let _response = menu.render(&mut contexts, &mut input_event_reader);
        }
    }
}

// End Systems

// Helper Structs
#[derive(Debug, Clone, Hash)]
pub enum InputError {
    InvalidFilename(String),
}

impl InputError {
    pub fn to_string(&self) -> String {
        match self {
            Self::InvalidFilename(s) => {
                format!("Invalid Filename: {}", s)
            }
        }
    }
}

// End Helper Structs

// Helper Functions
fn sanitize_map_name(s: &str) -> Result<String, InputError> {
    if is_sanitized(s) {
        return Ok(s.to_string());
    } else {
        Err(InputError::InvalidFilename(s.to_string()))
    }
}

// End Helper Functions
