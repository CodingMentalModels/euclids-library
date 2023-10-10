use bevy::prelude::*;

use super::{events::OpenMenuEvent, resources::GameState, ui_state::MenuUIState};

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
) {
    for event in open_menu_event_reader.iter() {
        commands.insert_resource(NextState(Some(GameState::Menu)));
        match event.0 {
            MenuType::Character => {
                let options = vec![
                    "Body".to_string(),
                    "Head".to_string(),
                    "Left Leg".to_string(),
                    "Right Leg".to_string(),
                ];
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
