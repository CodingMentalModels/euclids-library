use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::constants::*;
use crate::game::resources::*;

use crate::game::dialog::Dialog;
use crate::game::map::MapLocation;
use crate::game::map::TileLocation;
use crate::game::npc::NPC;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::LoadingAssets),
            generate_example_specs_system,
        )
        .add_systems(OnEnter(GameState::LoadingAssets), load_assets_system);
    }
}

// Systems

fn generate_example_specs_system() {
    let mut example_dialog = Dialog::story(
        "Example NPC".to_string(),
        vec!["I'm an NPC.".to_string(), "Hear me roar!".to_string()],
    );
    example_dialog.update_leaf_at(
        vec![],
        Dialog::PlayerDialog(Box::new(vec![
            ("I couldn't hear that.".to_string(), None),
            ("Ahhhhh!".to_string(), None),
        ])),
    );
    let npc = NPC::new(
        "Example NPC".to_string(),
        MapLocation::new(0, TileLocation::new(4, 5)),
        example_dialog,
    );

    let npc_string = serde_json::to_string(&npc).expect("Error serializing npc");

    fs::write(
        Path::new(NPC_DIRECTORY).join("example_npc.json"),
        npc_string,
    )
    .expect("Error writing npc spec");
}

fn load_assets_system(mut commands: Commands) {
    let file_loader: FileSystem<NPC> = FileSystem::new_directory(NPC_DIRECTORY);
    let npcs = file_loader
        .load_all()
        .expect("We should be able to load NPCs.")
        .into_iter()
        .map(|(_name, npc)| npc)
        .collect::<Vec<NPC>>();

    commands.insert_resource(NPCSpecs::from_vec(npcs));
    let rng = StdRng::seed_from_u64(12345);
    commands.insert_resource(RngResource(RwLock::new(rng)));
    commands.insert_resource(NextState(Some(GameState::LoadingUI)));
}

// End Systems

// Resources
#[derive(Debug, Resource)]
pub enum FileSystem<T: Clone + Serialize + DeserializeOwned> {
    Stub(HashMap<String, T>),
    Directory(Box<PathBuf>),
}

impl<T: Clone + Serialize + DeserializeOwned> FileSystem<T> {
    pub fn new_directory(directory_name: &str) -> Self {
        Self::Directory(Box::new(PathBuf::from(directory_name)))
    }
    pub fn load(&self, filename: &str) -> Result<T, FileSystemError> {
        self.load_all().and_then(|map| {
            map.get(filename)
                .cloned()
                .ok_or(FileSystemError::NoSuchFile)
        })
    }

    pub fn load_all(&self) -> Result<HashMap<String, T>, FileSystemError> {
        match self {
            Self::Stub(map) => Ok(map.clone()),
            Self::Directory(dir) => Self::read_file_names_and_files_from_directory(dir),
        }
    }

    fn read_file_names_and_files_from_directory(
        directory: &Path,
    ) -> Result<HashMap<String, T>, FileSystemError> {
        let paths = fs::read_dir(directory);
        let mut to_return = HashMap::new();
        match paths {
            Ok(read_dir) => {
                for subpath_result in read_dir {
                    match subpath_result {
                        Ok(dir_entry) => {
                            let subpath = dir_entry.path();
                            if subpath.is_file() {
                                let cloned_subpath = subpath.clone();
                                let filename = cloned_subpath.file_name();
                                let contents_result = fs::read_to_string(subpath);
                                match contents_result {
                                    Ok(contents) => {
                                        if contents.len() > 0 {
                                            to_return.insert(
                                            filename
                                                .expect("We know we're a file")
                                                .to_str()
                                                .expect(
                                                    "The filename should be a legitimate string.",
                                                )
                                                .to_string(),
                                            serde_json::from_str(&contents).unwrap(), // .map_err(|_| FileSystemError::CouldntDeserialize)?,
                                        );
                                        }
                                    }
                                    Err(_) => {
                                        return Err(FileSystemError::CouldntReadFile);
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            return Err(FileSystemError::CouldntReadSubpath);
                        }
                    }
                }
            }
            Err(_) => {
                return Err(FileSystemError::CouldntReadDirectory);
            }
        }

        return Ok(to_return);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FileSystemError {
    NoSuchFile,
    CouldntReadDirectory,
    CouldntReadSubpath,
    CouldntReadFile,
    CouldntDeserialize,
}

// End Resources

// Helper Functions

// End Helper Functions
