use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;
use egui::Ui;

use crate::constants::*;
use crate::game::resources::*;
use crate::input::MouseoverRaycastSet;

use crate::game::dialog::Dialog;
use crate::game::events::UpdateUIEvent;
use crate::game::interacting::{
    update_interacting_ui_state_system, Interactable, InteractingState, InteractingUIState,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        let generalized_exploring =
            || in_state(GameState::Exploring).or_else(in_state(GameState::NonPlayerTurns));
        app.add_event::<UpdateUIEvent>()
            .add_event::<ToastMessageEvent>()
            .add_plugins(EguiPlugin)
            .add_systems(OnEnter(GameState::LoadingUI), configure_visuals)
            .add_systems(OnEnter(GameState::LoadingUI), ui_load_system)
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
pub fn render_log(ui: &mut Ui, log_state: &LogState) {
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

pub fn get_egui_color(rgb: (f32, f32, f32)) -> egui::Color32 {
    let r = (rgb.0 * 256.) as u8;
    let g = (rgb.1 * 256.) as u8;
    let b = (rgb.2 * 256.) as u8;

    egui::Color32::from_rgb(r, g, b)
}
// End Helper Functions
