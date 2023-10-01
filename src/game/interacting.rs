use bevy::prelude::*;

use super::{
    dialog::Dialog,
    events::DirectionEvent,
    map::Map,
    player::{LocationComponent, PlayerComponent},
    resources::{GameState, LoadedMap},
};

pub struct InteractingPlugin;

impl Plugin for InteractingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_direction_choice_system
                .run_if(in_state(GameState::Interacting).and_then(on_event::<DirectionEvent>())),
        );
    }
}

// Resources

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Resource)]
enum InteractingState {
    #[default]
    ChoosingDirection,
    Interacting(Interactable),
}

// End Resources

// Components
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InteractableComponent(pub Interactable);

// End Components

// Systems

fn handle_direction_choice_system(
    mut state: ResMut<InteractingState>,
    mut reader: EventReader<DirectionEvent>,
    player_query: Query<&LocationComponent, With<PlayerComponent>>,
    interactable_query: Query<
        (&LocationComponent, &InteractableComponent),
        Without<PlayerComponent>,
    >,
) {
    let player_location = player_query.single();
    match *state {
        InteractingState::ChoosingDirection => {
            for direction_event in reader.iter() {
                let interact_location =
                    player_location.translated(direction_event.0.as_tile_location());
                match interactable_query
                    .iter()
                    .find(|(location, _interactable)| **location == interact_location)
                {
                    None => {
                        info!("Nothing to interact with.");
                    }
                    Some((_location, interaction)) => {
                        *state = InteractingState::Interacting(interaction.0.clone());
                    }
                }
            }
        }
        _ => {
            // Do nothing
        }
    }
}
// End Systems

// Structs
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Interactable {
    Dialog(Dialog),
}

// End Structs
