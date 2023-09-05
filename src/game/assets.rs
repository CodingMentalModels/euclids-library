use bevy::prelude::*;
use std::fs;
use std::path::Path;

use crate::game::constants::*;
use crate::game::resources::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {}
}

// Systems

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
                        if (subpath.is_file()) {
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
