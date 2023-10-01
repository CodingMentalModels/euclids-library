use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;

use crate::game::constants::*;
use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;

use super::events::CameraZoomEvent;
use super::interacting::{Interactable, InteractingState};
use super::npc::NPCComponent;
use super::player::{LocationComponent, PlayerComponent};
use super::ui_state::{
    AsciiTileAppearance, ExploringUIState, InteractingUIState, TileAppearance, TileGrid,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(OnEnter(GameState::LoadingUI), configure_visuals)
            .add_systems(OnEnter(GameState::LoadingUI), ui_load_system)
            .add_systems(OnEnter(GameState::Exploring), load_map)
            .add_systems(
                Update,
                update_camera_zoom.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(
                Update,
                update_positions.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(
                Update,
                render_exploring_ui.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(
                Update,
                render_interacting_ui.run_if(in_state(GameState::Interacting)),
            );

        app.insert_resource(MaterialCache::empty());
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
    let (_player_entity, player_location) = player_query.single();
    let map_layer = map
        .0
        .get_layer(player_location.0.get_map_layer())
        .expect("Player's layer must exist.");

    let tile_grid = TileGrid::from_map_layer(map_layer.clone());
    commands.insert_resource(ExploringUIState {
        tile_grid: tile_grid.clone(),
    });

    tile_grid.render(&mut commands, font.0.clone());
}

fn update_positions(mut query: Query<(&LocationComponent, &mut Transform)>) {
    for (location, mut transform) in query.iter_mut() {
        let screen_coordinates =
            TileGrid::tile_to_world_coordinates(location.0.get_tile_location());
        *transform = Transform::from_translation(screen_coordinates.extend(0.));
    }
}

fn update_camera_zoom(
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut zoom_event_reader: EventReader<CameraZoomEvent>,
) {
    // TODO Add bounds to the zoom
    for mut orthographic_projection in query.iter_mut() {
        for event in zoom_event_reader.iter() {
            let amount = -CAMERA_ZOOM_SPEED * (event.0 as f32);
            let log_zoom = orthographic_projection.scale
                * CAMERA_ZOOM_LOG_BASE.powf(CAMERA_ZOOM_SPEED * amount);
            orthographic_projection.scale = log_zoom;
        }
    }
}

fn render_exploring_ui(
    mut commands: Commands,
    ui_state: Res<ExploringUIState>,
    player_query: Query<(Entity, &LocationComponent), With<PlayerComponent>>,
    npc_query: Query<(Entity, &LocationComponent), With<NPCComponent>>,
    font: Res<LoadedFont>,
) {
    let (player_entity, player_location) = player_query.single();

    let player_tile = TileAppearance::Ascii(AsciiTileAppearance::from('@'));
    let player_sprite = player_tile.render(
        &mut commands
            .get_entity(player_entity)
            .expect("Player entity must exist if it was returned from the query."),
        font.0.clone(),
        TileGrid::tile_to_world_coordinates(player_location.0.get_tile_location()),
    );
    commands.entity(player_sprite).insert(PlayerSprite);

    for (entity, location) in npc_query.iter() {
        let npc_tile = TileAppearance::Ascii('&'.into());
        let _npc_sprite = npc_tile.render(
            &mut commands
                .get_entity(entity)
                .expect("NPC Entity must exist if it was returned from the query."),
            font.0.clone(),
            TileGrid::tile_to_world_coordinates(location.0.get_tile_location()),
        );
    }
}

fn render_interacting_ui(ui_state: Res<InteractingUIState>) {
    match &ui_state.interacting_state {
        InteractingState::ChoosingDirection => {
            info!("Choose a direction.");
        }
        InteractingState::Interacting(interaction) => match &interaction {
            Interactable::Dialog(dialog) => {
                info!("{:?}", dialog);
            }
        },
    }
}

// Components
#[derive(Component, Clone, Copy)]
struct PlayerSprite;

// End Components

// Helper Structs

// End Helper Structs

// Helper Functions

// End Helper Functions
