use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui::{Align2, Color32, Frame};

use crate::constants::*;
use crate::game::map::Map;
use crate::game::resources::GameState;

// Basic Flow:
//  1. Load existing or new?
//  2. (If new) pick a name
//  3. (If new) pick a size
//  4. Use vim-like keybindings to move around, insert, copy, etc.
//
//  Everything should save on any change
//  Recording macros should be possible
//  Block mode should create rectangular blocks rather than be line-based etc.

pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::EditingMap), initialize_map_editor_system)
            .add_systems(
                Update,
                render_map_editor_system.run_if(in_state(GameState::EditingMap)),
            );
    }
}

// Resources

#[derive(Debug, Default, PartialEq, Eq, Hash, Resource)]
pub enum MapEditorUIState {
    #[default]
    NewOrLoadMenu,
    ChooseSizeMenu,
    Editing(MapEditorEditingState),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MapEditorEditingState {
    filename: String,
    map: Map,
    current_layer: usize,
    mode: EditingMode,
}

impl MapEditorEditingState {
    pub fn new(filename: String, map: Map, current_layer: usize, mode: EditingMode) -> Self {
        Self {
            filename,
            map,
            current_layer,
            mode,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum EditingMode {
    #[default]
    Normal,
    Insert,
    Block,
}

// End Resources

// Systems

fn initialize_map_editor_system(mut commands: Commands) {
    commands.insert_resource(MapEditorUIState::default());
}

fn render_map_editor_system(mut contexts: EguiContexts, ui_state: ResMut<MapEditorUIState>) {
    let ctx = contexts.ctx_mut();
    match *ui_state {
        MapEditorUIState::NewOrLoadMenu => {
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

                    ui.label("NewOrLoadMenu");
                });
        }
        _ => {
            unimplemented!()
        }
    }
}

// End Systems
