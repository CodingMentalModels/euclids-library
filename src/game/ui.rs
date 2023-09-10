use bevy::window::PrimaryWindow;
use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;
use egui::{RichText, Ui};

use crate::game::constants::*;
use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;

use super::map::{MapLayer, SurfaceTile, Tile};
use super::world::{LocationComponent, PlayerComponent};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(OnEnter(GameState::LoadingUI), configure_visuals)
            .add_systems(OnEnter(GameState::LoadingUI), ui_load_system)
            .add_systems(OnEnter(GameState::Exploring), load_map);
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

    commands.insert_resource(NextState(Some(GameState::InitializingWorld)));
}

fn load_map(
    mut commands: Commands,
    player_query: Query<&LocationComponent, With<PlayerComponent>>,
    map: Res<LoadedMap>,
    font: Res<LoadedFont>,
) {
    let player_location = player_query.single();

    let map_layer = map
        .0
        .get(player_location.map_layer)
        .expect("Player's layer must exist.");

    map_layer
        .as_i_j_tile_vector()
        .into_iter()
        .for_each(|(i, j, tile)| {
            let x = (i * TILE_WIDTH) as f32;
            let y = (j * TILE_HEIGHT) as f32;
            info!("Rendering tile at {}, {}", x, y);
            TileAppearance::from_tile(tile).render(&mut commands, font.0.clone(), Vec2::new(x, y));
        });

    let player_tile = TileAppearance::Ascii('@');
    player_tile.render(&mut commands, font.0.clone(), player_location.location);
}

// Helper Structs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileAppearance {
    Ascii(char),
    Sprite(Handle<TextureAtlas>),
}

impl TileAppearance {
    pub fn from_tile(tile: &Tile) -> Self {
        match tile.get_top_of_stack() {
            Some(object_tile) => {
                panic!("Not implemented yet.");
            }
            None => match tile.get_surface() {
                SurfaceTile::Ground => TileAppearance::Ascii(MIDDLE_DOT),
                SurfaceTile::Wall => TileAppearance::Ascii('#'),
            },
        }
    }

    pub fn render(&self, commands: &mut Commands, font: Handle<Font>, location: Vec2) {
        match self {
            TileAppearance::Ascii(character) => {
                commands.spawn(TextBundle {
                    text: Text::from_section(
                        *character,
                        TextStyle {
                            font: font.clone(),
                            font_size: ASCII_TILE_FONT_SIZE,
                            color: Color::BLUE,
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(location.x),
                        bottom: Val::Px(location.y),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            TileAppearance::Sprite(_) => panic!("Not implemented yet."),
        }
    }
}

// End Helper Structs

// Helper Functions
//
// End Helper Functions
