use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::result::Result;

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::assets::FileSystem;
use crate::constants::*;
use crate::game::events::{DespawnBoundEntitiesEvent, MenuInputEvent};
use crate::game::map::MapLocation;
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
            .add_event::<AddTransactionEvent>()
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
            .add_systems(OnEnter(GameState::EditingMap), spawn_map_system)
            .add_systems(
                Update,
                add_transaction_and_save_system.run_if(
                    in_state(GameState::EditingMap).and_then(on_event::<AddTransactionEvent>()),
                ),
            );
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
            MapEditorMenuType::LoadMapMenu(maps) => MenuUIState::new(MenuType::SearchAndSelect(
                "Load Map".to_string(),
                maps.to_vec(),
            )),
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

#[derive(Debug, Resource)]
pub struct MapEditorEditingUIState {
    filename: String,
    store: TransactionStore,
    current_layer: usize,
    mode: EditingMode,
}

impl MapEditorEditingUIState {
    pub fn new(
        filename: String,
        store: TransactionStore,
        current_layer: usize,
        mode: EditingMode,
    ) -> Result<Self, TransactionStoreError> {
        let _current_map_layer = store.compile_layer(current_layer)?;
        Ok(Self {
            filename,
            store,
            current_layer,
            mode,
        })
    }

    pub fn get_filename(&self) -> String {
        self.filename.clone()
    }

    pub fn get_map(&self) -> Result<Map, TransactionStoreError> {
        self.store.compile()
    }

    pub fn get_current_map_layer(&self) -> Result<MapLayer, TransactionStoreError> {
        let map = self.get_map()?;
        map.get_layer(self.current_layer)
            .map_err(|_| TransactionStoreError::CurrentLayerDoesntExist)
            .cloned()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum EditingMode {
    #[default]
    Normal,
    Insert,
    Block,
}

#[derive(Debug, Resource)]
pub struct LoadedMaps(HashMap<String, Map>);
// End Resources

// Events
#[derive(Debug, Hash, PartialEq, Eq, Event)]
pub struct MapEditorSwitchMenuEvent(MapEditorMenuType);

#[derive(Debug, PartialEq, Eq, Event)]
pub struct AddTransactionEvent(Option<Transaction>);

// End Events

// Systems
fn initialize_map_editor_menu_system(mut commands: Commands) {
    commands.insert_resource(MapEditorMenuUIState::default());

    let filesystem = FileSystem::<Map>::new_directory(MAP_DIRECTORY);
    let maps = filesystem
        .load_all()
        .expect("Error parsing maps.")
        .into_iter()
        .map(|(name, map)| {
            (
                name.split('.')
                    .next()
                    .expect("All files have a extension")
                    .to_string(),
                map,
            )
        })
        .collect();
    commands.insert_resource(filesystem);
    commands.insert_resource(LoadedMaps(maps));
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
    loaded_maps: Res<LoadedMaps>,
    mut input_event_reader: EventReader<MenuInputEvent>,
    mut switch_menu_event_writer: EventWriter<MapEditorSwitchMenuEvent>,
    mut toast_message_event_writer: EventWriter<ToastMessageEvent>,
    mut add_transaction_event_writer: EventWriter<AddTransactionEvent>,
    mut ui_state: ResMut<MapEditorMenuUIState>,
    mut despawn_event_writer: EventWriter<DespawnBoundEntitiesEvent>,
) {
    let response = ui_state.render(&mut contexts, &mut input_event_reader);
    match &mut ui_state.menu_type {
        MapEditorMenuType::NewOrLoadMenu => {
            let maps = loaded_maps.0.keys().cloned().collect();
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
                    match MapEditorEditingUIState::new(
                        filename,
                        TransactionStore::from_snapshot(map),
                        current_layer,
                        EditingMode::Normal,
                    ) {
                        Ok(ui_state) => {
                            despawn_event_writer
                                .send(DespawnBoundEntitiesEvent(GameState::Exploring));
                            add_transaction_event_writer.send(AddTransactionEvent(None));
                            commands.insert_resource(ui_state);
                            commands.insert_resource(NextState(Some(GameState::EditingMap)));
                        }
                        Err(e) => toast_message_event_writer
                            .send(ToastMessageEvent(format!("TransactionStoreError: {:?}", e))),
                    };
                }
                Err(e) => {
                    toast_message_event_writer.send(ToastMessageEvent(e.to_string()));
                }
            },
            None => {}
        },
        MapEditorMenuType::LoadMapMenu(_maps) => match response {
            Some(map_name) => {
                let filename = format!("{}.json", map_name);
                let map = loaded_maps
                    .0
                    .get(&map_name)
                    .expect("The only options are what we loaded.");
                let current_layer = 0;
                match MapEditorEditingUIState::new(
                    filename,
                    TransactionStore::from_snapshot(map.clone()),
                    current_layer,
                    EditingMode::Normal,
                ) {
                    Ok(ui_state) => {
                        despawn_event_writer.send(DespawnBoundEntitiesEvent(GameState::Exploring));
                        add_transaction_event_writer.send(AddTransactionEvent(None));
                        commands.insert_resource(ui_state);
                        commands.insert_resource(NextState(Some(GameState::EditingMap)));
                    }
                    Err(e) => toast_message_event_writer
                        .send(ToastMessageEvent(format!("TransactionStoreError: {:?}", e))),
                };
            }
            None => {}
        },
    }
}

fn spawn_map_system(
    mut commands: Commands,
    ui_state: Res<MapEditorEditingUIState>,
    font: Res<LoadedFont>,
    mut toast_message_event_writer: EventWriter<ToastMessageEvent>,
) {
    let map_layer = match ui_state.get_current_map_layer() {
        Err(e) => {
            toast_message_event_writer
                .send(ToastMessageEvent(format!("Transaction error: {:?}", e)));
            return;
        }
        Ok(map_layer) => map_layer,
    };
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

fn add_transaction_and_save_system(
    mut ui_state: ResMut<MapEditorEditingUIState>,
    mut event_reader: EventReader<AddTransactionEvent>,
    mut toast_message_event_writer: EventWriter<ToastMessageEvent>,
    mut filesystem: ResMut<FileSystem<Map>>,
) {
    for event in event_reader.iter() {
        let AddTransactionEvent(maybe_transaction) = event;

        if maybe_transaction.is_some() {
            ui_state.store.add(maybe_transaction.clone().unwrap());
        }

        match &ui_state.store.compile() {
            Err(e) => {
                toast_message_event_writer.send(ToastMessageEvent(format!(
                    "Error compiling map from transactions: {:?}",
                    e
                )));
            }
            Ok(map) => match filesystem.save(&ui_state.get_filename(), map.clone()) {
                Err(e) => {
                    toast_message_event_writer
                        .send(ToastMessageEvent(format!("Error saving map: {:?}", e)));
                }
                Ok(()) => {
                    // Do nothing
                }
            },
        }
    }
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

#[derive(Clone, Debug)]
pub struct TransactionStore {
    transactions: Vec<Transaction>,
}

impl TransactionStore {
    pub fn new(transactions: Vec<Transaction>) -> Self {
        Self { transactions }
    }

    pub fn from_snapshot(map: Map) -> Self {
        Self::new(vec![Transaction::Snapshot(map)])
    }

    pub fn add(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn undo(&mut self) -> Result<(), TransactionStoreError> {
        if self.transactions.len() == 0 {
            Err(TransactionStoreError::NoTransactionsToUndo)
        } else {
            self.transactions.remove(self.transactions.len() - 1);
            Ok(())
        }
    }

    pub fn compile_layer(&self, layer_idx: usize) -> Result<MapLayer, TransactionStoreError> {
        self.compile()?
            .get_layer(layer_idx)
            .map_err(|e| TransactionStoreError::CurrentLayerDoesntExist)
            .cloned()
    }

    pub fn compile(&self) -> Result<Map, TransactionStoreError> {
        let (mut result, from_last_snapshot) = self.get_from_last_snapshot()?;
        for transaction in from_last_snapshot.iter() {
            Self::apply_transaction(&mut result, transaction);
        }
        Ok(result)
    }

    fn get_from_last_snapshot(&self) -> Result<(Map, Vec<Transaction>), TransactionStoreError> {
        let (snapshot_idx, snapshot) = self
            .transactions
            .iter()
            .enumerate()
            .rev()
            .find(|(i, t)| t.is_snapshot())
            .ok_or(TransactionStoreError::NoSnapshotTransaction)?;
        if let Transaction::Snapshot(map) = snapshot {
            let remaining_transactions = self.transactions[snapshot_idx..].to_vec();
            Ok((map.clone(), remaining_transactions))
        } else {
            unreachable!();
        }
    }

    fn apply_transaction(
        map: &mut Map,
        transaction: &Transaction,
    ) -> Result<(), TransactionStoreError> {
        match transaction {
            Transaction::Snapshot(new_map) => {
                *map = new_map.clone();
            }
            Transaction::Update(tile, location) => {
                map.update(tile.clone(), location.clone());
            }
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transaction {
    Snapshot(Map),
    Update(Tile, MapLocation),
}

impl Transaction {
    pub fn is_snapshot(&self) -> bool {
        if let Self::Snapshot(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TransactionStoreError {
    NoSnapshotTransaction,
    NoTransactionsToUndo,
    CurrentLayerDoesntExist,
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
        .contains(&['/', '\\', '<', '>', ':', '"', '|', '?', '*', '\0', '.'][..])
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

#[cfg(test)]
mod tests {

    use crate::game::map::TileLocation;

    use super::*;

    #[test]
    fn test_transactions_compile() {
        let map: Map = MapLayer::fill(10, 20, Tile::empty_ground()).into();
        let mut store = TransactionStore::from_snapshot(map);
        store.add(Transaction::Update(
            Tile::wall(),
            TileLocation::new(3, 5).into(),
        ));
        store.add(Transaction::Update(
            Tile::wall(),
            TileLocation::new(4, 5).into(),
        ));
        store.add(Transaction::Update(
            Tile::wall(),
            TileLocation::new(5, 5).into(),
        ));
        store.undo();

        let mut expected: Map = MapLayer::fill(10, 20, Tile::empty_ground()).into();
        expected.update(Tile::wall(), TileLocation::new(3, 5).into());
        expected.update(Tile::wall(), TileLocation::new(4, 5).into());

        assert_eq!(store.compile(), Ok(expected));
    }
}
