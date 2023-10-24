use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs;
use std::path::Path;
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
    let npcs = read_files_from_directory(Path::new(NPC_DIRECTORY))
        .into_iter()
        .filter(|s| s.len() > 0)
        .map(|s| serde_json::from_str(&s))
        .collect::<Result<Vec<NPC>, _>>()
        .expect("Error parsing npcs.");

    commands.insert_resource(NPCSpecs::from_vec(npcs));
    let rng = StdRng::seed_from_u64(12345);
    commands.insert_resource(RngResource(RwLock::new(rng)));
    commands.insert_resource(NextState(Some(GameState::LoadingUI)));
}

// End Systems

// Helper Functions

fn read_files_from_directory(directory: &Path) -> Vec<String> {
    let paths = fs::read_dir(directory);
    let mut to_return = Vec::new();
    match paths {
        Ok(read_dir) => {
            for subpath_result in read_dir {
                match subpath_result {
                    Ok(dir_entry) => {
                        let subpath = dir_entry.path();
                        if subpath.is_file() {
                            let contents_result = fs::read_to_string(subpath);
                            match contents_result {
                                Ok(contents) => {
                                    to_return.push(contents);
                                }
                                Err(e) => {
                                    panic!("Error reading file: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        panic!("Error reading subpath: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            panic!("Error reading directory: {}", e);
        }
    }

    return to_return;
}

// End Helper Functions
