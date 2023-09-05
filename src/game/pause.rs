use bevy::prelude::*;

use crate::game::input::{input_system, PauseUnpauseEvent};
use crate::game::resources::*;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        let get_pause_states_condition =
            || in_state(GameState::Paused).or_else(in_state(GameState::Paused));

        app.insert_resource(PausedState(GameState::Paused))
            .add_system(
                pause_system
                    .run_if(get_pause_states_condition())
                    .before(input_system),
            );
    }
}

// Components

// End Components

// Systems
fn pause_system(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    next_state: Res<NextState<GameState>>,
    mut pause_unpause_event_reader: EventReader<PauseUnpauseEvent>,
    mut paused_state: ResMut<PausedState>,
) {
    for _ in pause_unpause_event_reader.iter() {
        if current_state.get() == &GameState::Paused {
            commands.insert_resource(NextState(Some(paused_state.0)));
        } else {
            paused_state.0 = match next_state.0 {
                Some(state) => state,
                None => *current_state.get(),
            };
            commands.insert_resource(NextState(Some(GameState::Paused)));
        }
    }
}

// End Systems
