use bevy::window::PrimaryWindow;
use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;
use egui::{RichText, Ui};

use crate::game::constants::*;
use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;

use super::tiles::{Tile, TileGrid};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(OnEnter(GameState::LoadingUI), configure_visuals)
            .add_systems(OnEnter(GameState::LoadingUI), ui_load_system)
            .add_systems(OnEnter(GameState::Exploring), render_map);
    }
}

// Systems
fn configure_visuals(mut ctx: EguiContexts) {
    ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn ui_load_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Roboto-Regular.ttf");

    if asset_server.get_load_state(font.clone()) == LoadState::Failed {
        panic!(
            "Failed to load font: {:?}",
            asset_server.get_load_state(font.clone())
        );
    }

    commands.insert_resource(LoadedFont(font.clone()));

    commands
        .spawn(Camera2dBundle::default())
        .insert(RaycastSource::<MouseoverRaycastSet>::new());

    commands.insert_resource(NextState(Some(GameState::LoadingMap)));
}

fn render_map(mut commands: Commands, tile_grid: Res<TileGrid>, font: Res<LoadedFont>) {
    tile_grid
        .as_i_j_tile_vector()
        .into_iter()
        .for_each(|(i, j, tile)| {
            let x = i * TILE_WIDTH;
            let y = j * TILE_HEIGHT;
            info!("Rendering tile at {}, {}", x, y);
            match tile {
                Tile::Ascii(character) => {
                    commands.spawn(TextBundle {
                        text: Text::from_section(
                            *character,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: ASCII_TILE_FONT_SIZE,
                                color: Color::BLUE,
                            },
                        ),
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(x as f32),
                            bottom: Val::Px(y as f32),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                }
                Tile::Sprite(_) => panic!("Not implemented yet."),
            }
        });
}

// Helper Functions
//
// End Helper Functions
