use std::ffi::OsStr;
use std::path::Path;
use std::result::Result;

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::constants::*;
use crate::game::events::{DespawnBoundEntitiesEvent, MenuInputEvent};
use crate::game::map::{Map, MapLayer, Tile, TileGrid};
use crate::game::resources::{GameState, LoadedFont};
use crate::menu::{MenuType, MenuUIState};
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
        app.add_event::<MapEditorSwitchMenuEvent>()
            .add_systems(
                OnEnter(GameState::EditingMapMenu),
                initialize_map_editor_menu_system,
            )
            .add_systems(
                Update,
                render_map_editor_menu_system.run_if(in_state(GameState::EditingMapMenu)),
            )
            .add_systems(
                Update,
                switch_menu_system.run_if(
                    in_state(GameState::EditingMapMenu)
                        .and_then(on_event::<MapEditorSwitchMenuEvent>()),
                ),
            )
            .add_systems(OnEnter(GameState::EditingMap), spawn_map_system);
    }
}

// Resources
#[derive(Clone, Debug, PartialEq, Eq, Hash, Resource)]
pub struct MapEditorMenuUIState {
    menu_type: MapEditorMenuType,
    menu_state: MenuUIState,
}

impl Default for MapEditorMenuUIState {
    fn default() -> Self {
        Self::from_type(MapEditorMenuType::default())
    }
}

impl MapEditorMenuUIState {
    pub fn new(menu_type: MapEditorMenuType, menu_state: MenuUIState) -> Self {
        Self {
            menu_type,
            menu_state,
        }
    }

    pub fn from_type(menu_type: MapEditorMenuType) -> Self {
        let menu_state = match &menu_type {
            MapEditorMenuType::NewOrLoadMenu => MenuUIState::new(MenuType::SelectFinite(vec![
                "New Map".to_string(),
                "Load Map".to_string(),
            ])),
            MapEditorMenuType::NewMapNameMenu => {
                MenuUIState::new(MenuType::TextInput("Map Name:".to_string()))
            }
            MapEditorMenuType::NewMapSizeMenu(_buffer) => {
                MenuUIState::new(MenuType::TextInput("Map Size:".to_string()))
            }
            MapEditorMenuType::LoadMapMenu(_maps) => {
                MenuUIState::new(MenuType::Info(vec!["Load Map Menu".to_string()]))
            }
        };
        Self::new(menu_type, menu_state)
    }

    pub fn render(
        &mut self,
        contexts: &mut EguiContexts,
        input_reader: &mut EventReader<MenuInputEvent>,
    ) -> Option<String> {
        self.menu_state.render(contexts, input_reader)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum MapEditorMenuType {
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
#[derive(Debug, Hash, PartialEq, Eq, Event)]
pub struct MapEditorSwitchMenuEvent(MapEditorMenuType);

// End Events

// Systems
fn initialize_map_editor_menu_system(mut commands: Commands) {
    commands.insert_resource(MapEditorMenuUIState::default());
}

fn switch_menu_system(
    mut switch_menu_event_reader: EventReader<MapEditorSwitchMenuEvent>,
    mut ui_state: ResMut<MapEditorMenuUIState>,
) {
    for event in switch_menu_event_reader.iter() {
        *ui_state = MapEditorMenuUIState::from_type(event.0.clone());
    }
}

fn render_map_editor_menu_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut input_event_reader: EventReader<MenuInputEvent>,
    mut switch_menu_event_writer: EventWriter<MapEditorSwitchMenuEvent>,
    mut toast_message_event_writer: EventWriter<ToastMessageEvent>,
    mut ui_state: ResMut<MapEditorMenuUIState>,
    mut despawn_event_writer: EventWriter<DespawnBoundEntitiesEvent>,
) {
    let response = ui_state.render(&mut contexts, &mut input_event_reader);
    match &mut ui_state.menu_type {
        MapEditorMenuType::NewOrLoadMenu => {
            // TODO Load this from folder
            let maps = vec![];
            match response.as_deref() {
                Some("New Map") => switch_menu_event_writer
                    .send(MapEditorSwitchMenuEvent(MapEditorMenuType::NewMapNameMenu)),
                Some("Load Map") => switch_menu_event_writer.send(MapEditorSwitchMenuEvent(
                    MapEditorMenuType::LoadMapMenu(maps),
                )),
                Some(_) => panic!("There are only two options."),
                None => {}
            };
        }
        MapEditorMenuType::NewMapNameMenu => match response {
            Some(map_name) => match sanitize_map_name(&map_name) {
                Ok(name) => {
                    switch_menu_event_writer.send(MapEditorSwitchMenuEvent(
                        MapEditorMenuType::NewMapSizeMenu(name.to_string()),
                    ));
                }
                Err(e) => toast_message_event_writer.send(ToastMessageEvent(e.to_string())),
            },
            None => {}
        },
        MapEditorMenuType::NewMapSizeMenu(map_name) => match response {
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
                    despawn_event_writer.send(DespawnBoundEntitiesEvent(GameState::Exploring));
                    commands.insert_resource(ui_state);
                    commands.insert_resource(NextState(Some(GameState::EditingMap)));
                }
                Err(e) => {
                    toast_message_event_writer.send(ToastMessageEvent(e.to_string()));
                }
            },
            None => {}
        },
        MapEditorMenuType::LoadMapMenu(maps) => {
            unimplemented!()
        }
    }
}

fn spawn_map_system(
    mut commands: Commands,
    ui_state: Res<MapEditorEditingUIState>,
    font: Res<LoadedFont>,
) {
    let map_layer = ui_state
        .map
        .get_layer(ui_state.current_layer)
        .expect("Current layer must exist.");

    let tile_grid = TileGrid::from_map_layer(map_layer.clone());
    tile_grid.render(&mut commands, font.0.clone(), GameState::EditingMap);

    map_layer
        .as_location_and_tile_vector()
        .into_iter()
        .for_each(|(location, tile)| {
            if let Some(particle_spec) = tile.get_surface().get_particle_spec() {
                particle_spec.render(
                    &mut commands,
                    font.0.clone(),
                    GameState::EditingMap,
                    location,
                );
            };
        });
}

// End Systems

// Helper Structs
#[derive(Debug, Clone, Hash)]
pub enum InputError {
    InvalidFileStem(String),
}

impl InputError {
    pub fn to_string(&self) -> String {
        match self {
            Self::InvalidFileStem(s) => {
                format!("Invalid Filename: {}", s)
            }
        }
    }
}

// End Helper Structs

// Helper Functions
fn sanitize_map_name<S: AsRef<OsStr> + Into<String>>(name: S) -> Result<S, InputError> {
    let name_as_str = name.as_ref();

    // Empty name or too long for most file systems
    if name_as_str.is_empty() || name_as_str.len() > 255 {
        return Err(InputError::InvalidFileStem(name.into()));
    }

    // Contains characters not allowed in Unix and Windows filenames
    if name_as_str
        .to_string_lossy()
        .contains(&['/', '\\', '<', '>', ':', '"', '|', '?', '*', '\0'][..])
    {
        return Err(InputError::InvalidFileStem(name.into()));
    }

    // Reserved Windows filenames
    let lower_name = name_as_str.to_string_lossy().to_lowercase();
    let reserved_names = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    if reserved_names.contains(&lower_name.as_str()) {
        return Err(InputError::InvalidFileStem(name.into()));
    }

    // Leading or trailing spaces and periods not allowed on Windows
    if lower_name.starts_with(' ')
        || lower_name.ends_with(' ')
        || lower_name.starts_with('.')
        || lower_name.ends_with('.')
    {
        return Err(InputError::InvalidFileStem(name.into()));
    }

    // If all checks pass, the filename is valid
    Ok(name)
}
// End Helper Functions
