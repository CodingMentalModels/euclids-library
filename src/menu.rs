use bevy::prelude::*;

use crate::game::{
    character::BodyComponent, events::OpenMenuEvent, player::PlayerComponent, resources::GameState,
};

use crate::ui_state::MenuUIState;
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            open_menu_system.run_if(in_state(GameState::Exploring)),
        )
        .add_systems(OnExit(GameState::Menu), tear_down_menu_system);
    }
}

// Systems

fn open_menu_system(
    mut commands: Commands,
    mut open_menu_event_reader: EventReader<OpenMenuEvent>,
    player_query: Query<&BodyComponent, With<PlayerComponent>>,
) {
    for event in open_menu_event_reader.iter() {
        commands.insert_resource(NextState(Some(GameState::Menu)));
        match event.0 {
            MenuType::Character => {
                let player_body_component = player_query.single();
                let options = player_body_component.0.get_menu_text();
                commands.insert_resource(MenuUIState::new(options));
            }
        }
    }
}

fn tear_down_menu_system(mut commands: Commands) {
    commands.remove_resource::<MenuUIState>();
}
// End Systems

// Helper Structs
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MenuType {
    Character,
}
// End Helper Structs
