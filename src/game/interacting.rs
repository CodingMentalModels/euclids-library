use bevy::prelude::*;
use bevy_egui::EguiContexts;

use super::character::LocationComponent;
use super::{
    dialog::Dialog,
    events::{ChooseDirectionEvent, ProgressPromptEvent, UpdateUIEvent},
    player::PlayerComponent,
    resources::GameState,
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
                progress_prompt_system.run_if(
                    in_state(GameState::Interacting).and_then(on_event::<ProgressPromptEvent>()),
                ),
            )
            .add_systems(
                Update,
                render_interacting_ui
                    .after(update_interacting_ui_state_system)
                    .run_if(in_state(GameState::Interacting)),
            )
            .add_systems(OnExit(GameState::Interacting), tear_down_interacting_system);
    }
}

// Resources

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Resource)]
pub struct InteractingUIState {
    pub interacting_state: InteractingState,
}

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

fn progress_prompt_system(
    mut commands: Commands,
    mut ui_state: ResMut<InteractingUIState>,
    mut progress_prompt_event_reader: EventReader<ProgressPromptEvent>,
) {
    for progress_prompt_event in progress_prompt_event_reader.iter() {
        match &ui_state.interacting_state {
            InteractingState::Interacting(interaction) => match &interaction {
                Interactable::Dialog(dialog) => match dialog {
                    Dialog::PlayerDialog(options) => {
                        match progress_prompt_event {
                            ProgressPromptEvent::ChooseOption(option) => {
                                if *option >= options.len() {
                                    // Do nothing
                                } else {
                                    info!(
                                        "Progressing dialog with choice {}: {}.",
                                        option,
                                        options[*option].0.clone()
                                    );
                                    match &options[*option].1 {
                                        Some(next_dialog) => {
                                            ui_state.interacting_state =
                                                InteractingState::Interacting(
                                                    Interactable::Dialog(next_dialog.clone()),
                                                );
                                        }
                                        None => {
                                            commands.insert_resource(NextState(Some(
                                                GameState::Exploring,
                                            )));
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Do nothing
                            }
                        }
                    }

                    Dialog::NPCDialog(npc_dialog) => {
                        match progress_prompt_event {
                            ProgressPromptEvent::Continue => match npc_dialog.get_next() {
                                Some(next_dialog) => {
                                    info!("Progressing dialog.");
                                    ui_state.interacting_state = InteractingState::Interacting(
                                        Interactable::Dialog(next_dialog),
                                    );
                                }
                                None => {
                                    commands.insert_resource(NextState(Some(GameState::Exploring)));
                                }
                            },
                            _ => {
                                // Do nothing
                            }
                        }
                    }
                },
            },
            _ => {
                // Do nothing
            }
        }
    }
}

fn render_interacting_ui(mut contexts: EguiContexts, ui_state: ResMut<InteractingUIState>) {
    let ctx = contexts.ctx_mut();
    match &ui_state.interacting_state {
        InteractingState::ChoosingDirection => {
            egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
                ui.label("Choose a direction to interact.");
            });
        }
        InteractingState::Interacting(interaction) => match &interaction {
            Interactable::Dialog(dialog) => {
                let content = match dialog {
                    Dialog::PlayerDialog(options) => {
                        let content = options
                            .iter()
                            .enumerate()
                            .map(|(i, option)| format!("{}) {}", i, option.0))
                            .collect::<Vec<_>>();
                        content.join("\n")
                    }
                    Dialog::NPCDialog(npc_dialog) => {
                        format!(
                            "{}: {}",
                            npc_dialog.get_speaker(),
                            npc_dialog.get_contents()
                        )
                    }
                };

                egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
                    ui.label(content);
                });
            }
        },
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
