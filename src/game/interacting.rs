use bevy::prelude::*;

use super::{
    dialog::Dialog,
    events::{ChooseDirectionEvent, ContinueEvent, UpdateUIEvent},
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
                update_interacting_ui_state_system
                    .run_if(in_state(GameState::Interacting).and_then(on_event::<UpdateUIEvent>())),
            )
            .add_systems(
                Update,
                progress_prompt_system
                    .run_if(in_state(GameState::Interacting).and_then(on_event::<ContinueEvent>())),
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

fn setup_interacting_system(
    mut commands: Commands,
    mut update_ui_event_writer: EventWriter<UpdateUIEvent>,
) {
    commands.insert_resource(InteractingState::default());
    commands.insert_resource(InteractingUIState::default());
    update_ui_event_writer.send(UpdateUIEvent);
}

fn handle_direction_choice_system(
    mut commands: Commands,
    mut state: ResMut<InteractingState>,
    mut reader: EventReader<ChooseDirectionEvent>,
    mut update_ui_event_writer: EventWriter<UpdateUIEvent>,
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
                        update_ui_event_writer.send(UpdateUIEvent);
                    }
                }
            }
        }
        _ => {
            // Do nothing
        }
    }
}

pub fn update_interacting_ui_state_system(
    mut ui_state: ResMut<InteractingUIState>,
    interacting_state: Res<InteractingState>,
) {
    ui_state.interacting_state = interacting_state.clone();
}

fn progress_prompt_system(mut commands: Commands, mut ui_state: ResMut<InteractingUIState>) {
    match &ui_state.interacting_state {
        InteractingState::Interacting(interaction) => match &interaction {
            Interactable::Dialog(dialog) => match dialog {
                Dialog::PlayerDialog(options) => {
                    panic!("Not implemented yet.");
                }
                Dialog::NPCDialog(npc_dialog) => match npc_dialog.get_next() {
                    Some(next_dialog) => {
                        info!("Progressing dialog.");
                        ui_state.interacting_state =
                            InteractingState::Interacting(Interactable::Dialog(next_dialog));
                    }
                    None => {
                        commands.insert_resource(NextState(Some(GameState::Exploring)));
                    }
                },
            },
        },
        _ => {
            // Do nothing
        }
    }
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
