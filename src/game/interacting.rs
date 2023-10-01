use bevy::prelude::*;

use super::{
    dialog::Dialog,
    events::ChooseDirectionEvent,
    map::Map,
    player::{LocationComponent, PlayerComponent},
    resources::{GameState, LoadedMap},
    ui_state::{ExploringUIState, InteractingUIState},
};

pub struct InteractingPlugin;

impl Plugin for InteractingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Interacting), setup_interacting_system)
            .add_systems(
                Update,
                handle_direction_choice_system.run_if(
                    in_state(GameState::Interacting).and_then(on_event::<ChooseDirectionEvent>()),
                ),
            )
            .add_systems(
                Update,
                render_system.run_if(in_state(GameState::Interacting)),
            )
            .add_systems(OnExit(GameState::Interacting), tear_down_interacting_system);
    }
}

// Resources

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Resource)]
pub enum InteractingState {
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

fn setup_interacting_system(mut commands: Commands, exploring_ui_state: Res<ExploringUIState>) {
    commands.insert_resource(InteractingState::default());
    commands.insert_resource(InteractingUIState::from(exploring_ui_state.clone()));
}

fn handle_direction_choice_system(
    mut commands: Commands,
    mut state: ResMut<InteractingState>,
    mut reader: EventReader<ChooseDirectionEvent>,
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
                        commands.insert_resource(NextState(Some(GameState::Exploring)));
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

fn render_system(
    mut ui_state: ResMut<InteractingUIState>,
    interacting_state: Res<InteractingState>,
) {
    ui_state.interacting_state = interacting_state.clone();
}

fn tear_down_interacting_system(mut commands: Commands) {
    commands.remove_resource::<InteractingState>();
    commands.remove_resource::<InteractingUIState>();
}

// End Systems

// Structs
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Interactable {
    Dialog(Dialog),
}

// End Structs
