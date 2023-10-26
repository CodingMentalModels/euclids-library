use std::unimplemented;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui::{Align2, Color32, Frame, RichText};

use crate::constants::*;
use crate::game::events::MenuInputEvent;
use crate::game::{
    character::BodyComponent, events::OpenMenuEvent, player::PlayerComponent, resources::GameState,
};
use crate::ui::get_default_text;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            open_menu_system.run_if(in_state(GameState::Exploring)),
        )
        .add_systems(Update, render_menu.run_if(in_state(GameState::PlayerMenu)))
        .add_systems(OnExit(GameState::PlayerMenu), tear_down_menu_system);
    }
}

// Resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct MenuToShow(pub MenuUIState);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuUIState {
    menu_type: MenuType,
}

impl MenuUIState {
    pub fn new(menu_type: MenuType) -> Self {
        Self { menu_type }
    }

    pub fn render(
        &self,
        mut contexts: &mut EguiContexts,
        mut input_reader: &mut EventReader<MenuInputEvent>,
    ) -> Option<&str> {
        let mut ctx = contexts.ctx_mut();
        match &self.menu_type {
            MenuType::Info(lines) => {
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

                        ui.label(get_default_text(lines.join("\n")));
                    });
                return None;
            }
            MenuType::SelectFinite(options) => {
                unimplemented!()
            }
            MenuType::SearchAndSelect => {
                unimplemented!()
            }
            MenuType::TextInput => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuType {
    Info(Vec<String>),
    SelectFinite(Vec<String>),
    SearchAndSelect,
    TextInput,
}

// End Resources

// Systems

fn open_menu_system(
    mut commands: Commands,
    mut open_menu_event_reader: EventReader<OpenMenuEvent>,
    player_query: Query<&BodyComponent, With<PlayerComponent>>,
) {
    for event in open_menu_event_reader.iter() {
        commands.insert_resource(NextState(Some(GameState::PlayerMenu)));
        match event.0 {
            ExploringMenuType::Character => {
                let player_body_component = player_query.single();
                let options = player_body_component.0.get_menu_text();
                commands.insert_resource(MenuToShow(MenuUIState::new(MenuType::Info(options))));
            }
        }
    }
}

fn render_menu(
    mut contexts: EguiContexts,
    mut event_reader: EventReader<MenuInputEvent>,
    menu: Res<MenuToShow>,
) {
    let _response = menu.0.render(&mut contexts, &mut event_reader);
}

fn tear_down_menu_system(mut commands: Commands) {
    commands.remove_resource::<MenuToShow>();
}
// End Systems

// Helper Structs
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExploringMenuType {
    Character,
}
// End Helper Structs
