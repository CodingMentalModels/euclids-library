use bevy::ecs::system::EntityCommands;
use bevy::window::PrimaryWindow;
use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;
use egui::{RichText, Ui};

use crate::game::constants::*;
use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;

use super::map::{MapLayer, SurfaceTile, Tile, TileLocation};
use super::world::{LocationComponent, PlayerComponent};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(OnEnter(GameState::LoadingUI), configure_visuals)
            .add_systems(OnEnter(GameState::LoadingUI), ui_load_system)
            .add_systems(OnEnter(GameState::Exploring), load_map)
            .add_systems(
                Update,
                update_text_positions.run_if(in_state(GameState::Exploring)),
            );
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
    player_query: Query<(Entity, &LocationComponent), With<PlayerComponent>>,
    map: Res<LoadedMap>,
    font: Res<LoadedFont>,
) {
    let (player_entity, player_location) = player_query.single();

    let map_layer = map
        .0
        .get(player_location.map_layer)
        .expect("Player's layer must exist.");

    map_layer
        .as_location_and_tile_vector()
        .into_iter()
        .for_each(|(tile_location, tile)| {
            TileAppearance::from_tile(tile).render(
                commands.spawn_empty(),
                font.0.clone(),
                to_screen_coordinates(tile_location),
            );
        });

    let player_tile = TileAppearance::Ascii('@');
    let player_sprite = player_tile.render(
        commands
            .get_entity(player_entity)
            .expect("Player entity must exist if it was returned from the query."),
        font.0.clone(),
        to_screen_coordinates(player_location.location),
    );
    commands.entity(player_sprite).insert(PlayerSprite);
}

fn update_text_positions(mut query: Query<(&LocationComponent, &mut Style), With<Text>>) {
    for (location, mut style) in query.iter_mut() {
        let screen_coordinates = to_screen_coordinates(location.location);
        style.left = Val::Px(screen_coordinates.x);
        style.bottom = Val::Px(screen_coordinates.y);
    }
}

// Components
#[derive(Component, Clone, Copy)]
struct PlayerSprite;

// End Components

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

    pub fn render(
        &self,
        mut entity_commands: EntityCommands,
        font: Handle<Font>,
        location: Vec2,
    ) -> Entity {
        info!("Rendering tile at {}", location);
        match self {
            TileAppearance::Ascii(character) => entity_commands
                .insert(TextBundle {
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
                })
                .id(),
            TileAppearance::Sprite(_) => panic!("Not implemented yet."),
        }
    }
}

// End Helper Structs

// Helper Functions

fn to_screen_coordinates(location: TileLocation) -> Vec2 {
    Vec2::new(
        (location.i * (TILE_WIDTH as i32)) as f32,
        (location.j * (TILE_HEIGHT as i32)) as f32,
    )
}
// End Helper Functions
