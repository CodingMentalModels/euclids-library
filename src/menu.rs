use std::collections::HashMap;
use std::unimplemented;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui::{Align2, Color32, Frame, Response, RichText, Ui};

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
            open_exploring_menu_system.run_if(in_state(GameState::Exploring)),
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
    buffer: String,
}

impl MenuUIState {
    pub fn new(menu_type: MenuType) -> Self {
        Self {
            menu_type,
            buffer: "".to_string(),
        }
    }

    pub fn render(
        &mut self,
        contexts: &mut EguiContexts,
        input_reader: &mut EventReader<MenuInputEvent>,
    ) -> Option<String> {
        let mut buffer = self.buffer.clone();
        match &self.menu_type {
            MenuType::Info(lines) => {
                self.display_content(contexts, lines.join("\n"));
                return None;
            }
            MenuType::SelectFinite(options) => {
                let labelled_options = options.iter().enumerate().collect::<HashMap<_, _>>();
                self.display_content(
                    contexts,
                    options
                        .iter()
                        .enumerate()
                        .map(|(i, option)| format!("{}. {}", i, option))
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
                for input_event in input_reader.iter() {
                    match get_digit_from_keycode(input_event.0) {
                        None => {}
                        Some(digit) => match labelled_options.get(&digit) {
                            Some(option) => return Some(option.to_string()),
                            None => {}
                        },
                    };
                }
                return None;
            }
            MenuType::SearchAndSelect => {
                unimplemented!()
            }
            MenuType::TextInput(prompt) => {
                let to_show_fn = |ui: &mut Ui| {
                    ui.label(get_default_text(prompt.to_string()));
                    let response = ui.add(egui::TextEdit::singleline(&mut buffer));
                    response.request_focus();
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let to_return = Some(buffer.clone());
                        return to_return;
                    } else {
                        return None;
                    }
                };
                let to_return = self.display(contexts, to_show_fn);
                self.buffer = buffer;
                return to_return;
            }
        }
    }

    fn display(
        &self,
        contexts: &mut EguiContexts,
        to_show_fn: impl FnOnce(&mut Ui) -> Option<String>,
    ) -> Option<String> {
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

                return to_show_fn(ui);
            })
            .and_then(|inner_response| inner_response.inner)
            .flatten()
    }

    fn display_content(&self, contexts: &mut EguiContexts, content: String) {
        let to_show_fn = |ui: &mut Ui| {
            ui.label(get_default_text(content));
            return None;
        };
        self.display(contexts, to_show_fn);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuType {
    Info(Vec<String>),
    SelectFinite(Vec<String>),
    SearchAndSelect,
    TextInput(String),
}

// End Resources

// Systems

fn open_exploring_menu_system(
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
    mut menu: ResMut<MenuToShow>,
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

// Helper Functions

fn get_digit_from_keycode(keycode: KeyCode) -> Option<usize> {
    match keycode {
        KeyCode::Key0 => Some(0),
        KeyCode::Key1 => Some(1),
        KeyCode::Key2 => Some(2),
        KeyCode::Key3 => Some(3),
        KeyCode::Key4 => Some(4),
        KeyCode::Key5 => Some(5),
        KeyCode::Key6 => Some(6),
        KeyCode::Key7 => Some(7),
        KeyCode::Key8 => Some(8),
        KeyCode::Key9 => Some(9),
        _ => None,
    }
}
// End Helper Functions
