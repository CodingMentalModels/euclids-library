use bevy::window::PrimaryWindow;
use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;
use egui::{RichText, Ui};

use crate::game::constants::*;
use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {}
}

// Systems
fn configure_visuals(mut ctx: EguiContexts) {
    ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

// Helper Functions
//
// End Helper Functions
