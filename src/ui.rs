use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;
use egui::{Align2, Color32, Frame, Ui};

use crate::constants::*;
use crate::game::resources::*;
use crate::input::MouseoverRaycastSet;

use crate::game::dialog::Dialog;
use crate::game::events::{CameraZoomEvent, UpdateUIEvent};
use crate::game::interacting::{
    update_interacting_ui_state_system, Interactable, InteractingState, InteractingUIState,
};
use crate::menu::MenuUIState;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        let generalized_exploring =
            || in_state(GameState::Exploring).or_else(in_state(GameState::NonPlayerTurns));
        let camera_zoom_states =
            || generalized_exploring().or_else(in_state(GameState::EditingMap));
        app.add_event::<UpdateUIEvent>()
            .add_event::<ToastMessageEvent>()
            .add_plugins(EguiPlugin)
            .add_systems(OnEnter(GameState::LoadingUI), configure_visuals)
            .add_systems(OnEnter(GameState::LoadingUI), ui_load_system)
            .add_systems(Update, update_camera_zoom.run_if(camera_zoom_states()))
            .add_systems(Update, render_exploring_ui.run_if(generalized_exploring()))
            .add_systems(
                Update,
                render_interacting_ui
                    .after(update_interacting_ui_state_system)
                    .run_if(in_state(GameState::Interacting)),
            )
            .add_systems(
                Update,
                trigger_toast_message.run_if(on_event::<ToastMessageEvent>()),
            )
            .add_systems(Update, show_toast_message);

        app.insert_resource(MaterialCache::empty());
    }
}

// Events

#[derive(Debug, Clone, Hash, PartialEq, Eq, Event)]
pub struct ToastMessageEvent(pub String);

// End Events

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

fn trigger_toast_message(mut commands: Commands, mut event_reader: EventReader<ToastMessageEvent>) {
    for event in event_reader.iter() {
        commands.insert_resource(ToastMessage(
            Timer::from_seconds(TOAST_MESSAGE_TIME_IN_SECONDS, TimerMode::Once),
            event.0.clone(),
        ));
    }
}

fn show_toast_message(
    mut commands: Commands,
    mut contexts: EguiContexts,
    time: Res<Time>,
    toast_message: Option<ResMut<ToastMessage>>,
) {
    match toast_message {
        Some(mut toast_message) => {
            let ctx = contexts.ctx_mut();
            egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
                ui.label(get_warning_text(toast_message.1.clone()));
            });
            if toast_message.0.tick(time.delta()).finished() {
                commands.remove_resource::<ToastMessage>();
            }
        }
        None => {}
    }
}

fn render_exploring_ui(mut contexts: EguiContexts, log_state: Res<LogState>) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
        egui::SidePanel::left("log-panel")
            .min_width(LOG_WINDOW_SIZE.0)
            .show_inside(ui, |ui| {
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

// End Systems

// Resources
#[derive(Resource, Clone, Default)]
pub struct ToastMessage(Timer, String);

#[derive(Resource, Clone, Default)]
pub struct LogState(Vec<egui::RichText>);

impl LogState {
    pub fn get_messages(&self) -> Vec<egui::RichText> {
        self.0.clone()
    }

    pub fn log_string(&mut self, message: &str) {
        self.0.push(
            egui::RichText::new(message.to_string())
                .size(LOG_TEXT_SIZE)
                .color(egui::Color32::WHITE),
        );
    }

    pub fn log_string_color(&mut self, message: &str, color: egui::Color32) {
        self.0.push(
            egui::RichText::new(message.to_string())
                .color(color)
                .size(LOG_TEXT_SIZE),
        );
    }

    pub fn log(&mut self, message: egui::RichText) {
        self.0.push(message);
    }
}
// End Resources

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

pub fn get_underlined_text(s: String) -> egui::RichText {
    get_default_text(s).underline()
}

pub fn get_default_text(s: String) -> egui::RichText {
    egui::RichText::new(s)
        .size(DEFAULT_FONT_SIZE)
        .color(egui::Color32::WHITE)
}

pub fn get_warning_text(s: String) -> egui::RichText {
    egui::RichText::new(s)
        .size(DEFAULT_FONT_SIZE)
        .color(egui::Color32::RED)
}
// End Helper Functions
