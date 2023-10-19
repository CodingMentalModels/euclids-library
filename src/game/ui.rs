use std::unimplemented;

use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;
use egui::{Align2, Color32, Frame, Ui};

use crate::game::constants::*;
use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;

use super::dialog::Dialog;
use super::events::{CameraZoomEvent, UpdateUIEvent};
use super::interacting::{update_interacting_ui_state_system, Interactable, InteractingState};
use super::ui_state::{InteractingUIState, LogState, MenuUIState};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        let generalized_exploring =
            || in_state(GameState::Exploring).or_else(in_state(GameState::NonPlayerTurns));
        app.add_event::<UpdateUIEvent>()
            .add_plugins(EguiPlugin)
            .add_systems(OnEnter(GameState::LoadingUI), configure_visuals)
            .add_systems(OnEnter(GameState::LoadingUI), ui_load_system)
            .add_systems(
                Update,
                update_camera_zoom.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(Update, render_exploring_ui.run_if(generalized_exploring()))
            .add_systems(
                Update,
                render_interacting_ui
                    .after(update_interacting_ui_state_system)
                    .run_if(in_state(GameState::Interacting)),
            )
            .add_systems(Update, render_menu_ui.run_if(in_state(GameState::Menu)));

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

    let mut log = LogState::default();
    commands.insert_resource(log);

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

fn render_exploring_ui(mut contexts: EguiContexts, log_state: Res<LogState>) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
        egui::SidePanel::left("log-panel")
            .min_width(LOG_WINDOW_SIZE.0)
            .show_inside(ui, |mut ui| {
                render_log(ui, &log_state);
            });
    });
}

fn render_interacting_ui(mut contexts: EguiContexts, ui_state: ResMut<InteractingUIState>) {
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

fn render_menu_ui(mut contexts: EguiContexts, ui_state: ResMut<MenuUIState>) {
    let ctx = contexts.ctx_mut();
    let size = egui::Vec2::new(ctx.screen_rect().width(), ctx.screen_rect().height())
        * MENU_TO_SCREEN_RATIO;
    egui::Window::new("menu-area")
        .anchor(
            Align2::CENTER_TOP,
            egui::Vec2::new(
                0.,
                (ctx.screen_rect().height() * (1. - MENU_TO_SCREEN_RATIO) / 2.),
            ),
        )
        .fixed_size(size)
        .frame(Frame::none().fill(Color32::BLACK))
        .title_bar(false)
        .show(ctx, |ui| {
            //  Workaround for https://users.rust-lang.org/t/egui-questions-regarding-window-size/88753/3
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            ui.label(ui_state.to_text());
        });
}

// Components

// End Components

// Helper Structs

// End Helper Structs

// Helper Functions
fn render_log(ui: &mut Ui, log_state: &LogState) {
    ui.label(get_underlined_text("Log".to_string()).size(LOG_TEXT_SIZE));
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .min_scrolled_height(LOG_WINDOW_SIZE.1)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for log_message in log_state.get_messages().into_iter() {
                ui.label(log_message);
            }
        });
}

fn get_underlined_text(s: String) -> egui::RichText {
    get_default_text(s).underline()
}

fn get_default_text(s: String) -> egui::RichText {
    egui::RichText::new(s)
        .size(DEFAULT_FONT_SIZE)
        .color(egui::Color32::WHITE)
}

// End Helper Functions
