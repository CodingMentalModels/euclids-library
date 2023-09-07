use bevy::window::PrimaryWindow;
use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_raycast::RaycastSource;
use egui::{RichText, Ui};

use crate::game::constants::*;
use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;

use super::tiles::TileGrid;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_map.run_if(in_state(GameState::Exploring)));
    }
}

// Systems
fn configure_visuals(mut ctx: EguiContexts) {
    ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn render_map(mut commands: Commands, tile_grid: Res<TileGrid>, font: Handle<Font>) {
    tile_grid.as_iter().for_each(|i, j, tile| {
        let x = i * TILE_WIDTH;
        let y = j * TILE_HEIGHT;
        commands.spawn(TextBundle::from_sections([TextSection::new(
            MIDDLE_DOT,
            TextStyle {
                font: font.clone(),
                font_size: ASCII_TILE_FONT_SIZE,
                color: Color::BLUE,
            },
        )]));
    })
}

// Helper Functions
//
// End Helper Functions
