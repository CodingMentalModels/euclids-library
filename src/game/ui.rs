use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;

use crate::game::constants::*;
use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;

use super::dialog::Dialog;
use super::events::{CameraZoomEvent, UpdateUIEvent};
use super::interacting::{update_interacting_ui_state_system, Interactable, InteractingState};
use super::ui_state::{
    AsciiTileAppearance, ExploringUIState, InteractingUIState, TileAppearance, TileGrid,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateUIEvent>()
            .add_plugins(EguiPlugin)
            .add_systems(OnEnter(GameState::LoadingUI), configure_visuals)
            .add_systems(OnEnter(GameState::LoadingUI), ui_load_system)
            .add_systems(
                Update,
                update_camera_zoom.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(
                Update,
                render_interacting_ui
                    .after(update_interacting_ui_state_system)
                    .run_if(in_state(GameState::Interacting)),
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

fn render_interacting_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut ui_state: ResMut<InteractingUIState>,
) {
    let ctx = contexts.ctx_mut();
    match &ui_state.interacting_state {
        InteractingState::ChoosingDirection => {
            egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
                ui.label("Choose a direction to interact.");
            });
        }
        InteractingState::Interacting(interaction) => match &interaction {
            Interactable::Dialog(dialog) => {
                let content = match dialog {
                    Dialog::PlayerDialog(options) => {
                        let content = options
                            .iter()
                            .enumerate()
                            .map(|(i, option)| format!("{}) {}", i, option.0))
                            .collect::<Vec<_>>();
                        content.join("\n")
                    }
                    Dialog::NPCDialog(npc_dialog) => {
                        format!(
                            "{}: {}",
                            npc_dialog.get_speaker(),
                            npc_dialog.get_contents()
                        )
                    }
                };

                egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
                    ui.label(content);
                });
            }
        },
    }
}

// Components

// End Components

// Helper Structs

// End Helper Structs

// Helper Functions

// End Helper Functions
