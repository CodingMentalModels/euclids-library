use bevy::prelude::*;
use bevy_mod_raycast::{
    print_intersections, DefaultRaycastingPlugin, RaycastMethod, RaycastSource, RaycastSystem,
};

use super::events::{
    CameraMovementEvent, CameraZoomEvent, ChooseDirectionEvent, Direction, MovementEvent,
    OpenMenuEvent, ProgressPromptEvent, StateChangeEvent,
};
use super::menu::MenuType;
use super::resources::GameState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PauseUnpauseEvent>()
            .add_event::<OpenMenuEvent>()
            .add_event::<CameraMovementEvent>()
            .add_event::<CameraZoomEvent>()
            .add_event::<MovementEvent>()
            .add_event::<StateChangeEvent>()
            .add_event::<ChooseDirectionEvent>()
            .add_event::<ProgressPromptEvent>()
            .add_systems(
                First,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays::<MouseoverRaycastSet>),
            )
            //.add_systems(Update, print_intersections::<MouseoverRaycastSet>)
            .add_systems(Update, input_system);
    }
}

// Components

// End Components

// Events

#[derive(Debug, Clone, Copy, PartialEq, Eq, Event)]
pub struct PauseUnpauseEvent;

// End Events

// Systems
pub fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    state: Res<State<GameState>>,
    mut pause_unpause_event_writer: EventWriter<PauseUnpauseEvent>,
    state_change_event_writer: EventWriter<StateChangeEvent>,
    camera_movement_event_writer: EventWriter<CameraMovementEvent>,
    zoom_event_writer: EventWriter<CameraZoomEvent>,
    open_menu_event_writer: EventWriter<OpenMenuEvent>,
    movement_event_writer: EventWriter<MovementEvent>,
    choose_direction_event_writer: EventWriter<ChooseDirectionEvent>,
    progress_prompt_event_writer: EventWriter<ProgressPromptEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        pause_unpause_event_writer.send(PauseUnpauseEvent);
    }

    match state.get() {
        GameState::Exploring => {
            handle_camera_movement(&keyboard_input, camera_movement_event_writer);
            handle_camera_zoom(&keyboard_input, zoom_event_writer);
            handle_movement(&keyboard_input, movement_event_writer);
            handle_interact(&keyboard_input, state_change_event_writer);
            handle_open_menu(&keyboard_input, open_menu_event_writer);
        }
        GameState::Interacting => {
            handle_progress_prompt(&keyboard_input, progress_prompt_event_writer);
            handle_exit(
                &keyboard_input,
                state_change_event_writer,
                GameState::Exploring,
            );
            handle_choose_direction(&keyboard_input, choose_direction_event_writer);
        }
        GameState::Menu => {
            handle_exit(
                &keyboard_input,
                state_change_event_writer,
                GameState::Exploring,
            );
        }
        _ => {}
    }
}

// Update our `RaycastSource` with the current cursor position every frame.
fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<MouseoverRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let Some(cursor_moved) = cursor.iter().last() else {
        return;
    };
    for mut pick_source in &mut query {
        pick_source.cast_method = RaycastMethod::Screenspace(cursor_moved.position);
    }
}
// End Systems

// Structs
#[derive(Debug, Clone, Reflect)]
pub struct MouseoverRaycastSet;

// End Structs

// Helper Functions

fn handle_movement(
    keyboard_input: &Res<Input<KeyCode>>,
    mut movement_event_writer: EventWriter<MovementEvent>,
) {
    match get_direction_from_keycode(keyboard_input) {
        Some(direction) => {
            movement_event_writer.send(MovementEvent(direction));
        }
        _ => {
            // Do nothing
        }
    }
}

fn handle_camera_zoom(
    keyboard_input: &Res<Input<KeyCode>>,
    mut zoom_event_writer: EventWriter<CameraZoomEvent>,
) {
    if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && keyboard_input.just_pressed(KeyCode::Equals)
    {
        zoom_event_writer.send(CameraZoomEvent(1));
    }

    if keyboard_input.just_pressed(KeyCode::Minus) {
        zoom_event_writer.send(CameraZoomEvent(-1));
    }
}

fn handle_camera_movement(
    keyboard_input: &Res<Input<KeyCode>>,
    mut camera_movement_event_writer: EventWriter<CameraMovementEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::W) {
        camera_movement_event_writer.send(CameraMovementEvent(Direction::Up));
    }
    if keyboard_input.just_pressed(KeyCode::A) {
        camera_movement_event_writer.send(CameraMovementEvent(Direction::Left));
    }
    if keyboard_input.just_pressed(KeyCode::S) {
        camera_movement_event_writer.send(CameraMovementEvent(Direction::Down));
    }
    if keyboard_input.just_pressed(KeyCode::D) {
        camera_movement_event_writer.send(CameraMovementEvent(Direction::Right));
    }
}

fn handle_interact(
    keyboard_input: &Res<Input<KeyCode>>,
    mut state_change_event_writer: EventWriter<StateChangeEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state_change_event_writer.send(StateChangeEvent(GameState::Interacting));
    }
}

fn handle_open_menu(
    keyboard_input: &Res<Input<KeyCode>>,
    mut open_menu_event_writer: EventWriter<OpenMenuEvent>,
) {
    if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && keyboard_input.just_pressed(KeyCode::Key2)
    {
        open_menu_event_writer.send(OpenMenuEvent(MenuType::Character));
    }
}

fn handle_exit(
    keyboard_input: &Res<Input<KeyCode>>,
    mut state_change_event_writer: EventWriter<StateChangeEvent>,
    state: GameState,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state_change_event_writer.send(StateChangeEvent(state));
    }
}

fn handle_progress_prompt(
    keyboard_input: &Res<Input<KeyCode>>,
    mut progress_prompt_event_writer: EventWriter<ProgressPromptEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::Return) {
        progress_prompt_event_writer.send(ProgressPromptEvent::Continue);
    } else if let Some(digit) = get_digit_from_keycode(keyboard_input) {
        progress_prompt_event_writer.send(ProgressPromptEvent::ChooseOption(digit));
    }
}

fn handle_choose_direction(
    keyboard_input: &Res<Input<KeyCode>>,
    mut choose_direction_event_writer: EventWriter<ChooseDirectionEvent>,
) {
    match get_direction_from_keycode(keyboard_input) {
        Some(direction) => {
            choose_direction_event_writer.send(ChooseDirectionEvent(direction));
        }
        _ => {
            // Do nothing
        }
    }
}

fn get_direction_from_keycode(keyboard_input: &Res<Input<KeyCode>>) -> Option<Direction> {
    if keyboard_input.just_pressed(KeyCode::Numpad8) {
        Some(Direction::Up)
    } else if keyboard_input.just_pressed(KeyCode::Numpad2) {
        Some(Direction::Down)
    } else if keyboard_input.just_pressed(KeyCode::Numpad4) {
        Some(Direction::Left)
    } else if keyboard_input.just_pressed(KeyCode::Numpad6) {
        Some(Direction::Right)
    } else if keyboard_input.just_pressed(KeyCode::Numpad7) {
        Some(Direction::UpLeft)
    } else if keyboard_input.just_pressed(KeyCode::Numpad9) {
        Some(Direction::UpRight)
    } else if keyboard_input.just_pressed(KeyCode::Numpad1) {
        Some(Direction::DownLeft)
    } else if keyboard_input.just_pressed(KeyCode::Numpad3) {
        Some(Direction::DownRight)
    } else {
        None
    }
}

fn get_digit_from_keycode(keyboard_input: &Res<Input<KeyCode>>) -> Option<usize> {
    if keyboard_input.just_pressed(KeyCode::Numpad0) {
        Some(0)
    } else if keyboard_input.just_pressed(KeyCode::Numpad1) {
        Some(1)
    } else if keyboard_input.just_pressed(KeyCode::Numpad2) {
        Some(2)
    } else if keyboard_input.just_pressed(KeyCode::Numpad3) {
        Some(3)
    } else if keyboard_input.just_pressed(KeyCode::Numpad4) {
        Some(4)
    } else if keyboard_input.just_pressed(KeyCode::Numpad5) {
        Some(5)
    } else if keyboard_input.just_pressed(KeyCode::Numpad6) {
        Some(6)
    } else if keyboard_input.just_pressed(KeyCode::Numpad7) {
        Some(7)
    } else if keyboard_input.just_pressed(KeyCode::Numpad8) {
        Some(8)
    } else if keyboard_input.just_pressed(KeyCode::Numpad9) {
        Some(9)
    } else if keyboard_input.just_pressed(KeyCode::Key0) {
        Some(0)
    } else if keyboard_input.just_pressed(KeyCode::Key1) {
        Some(1)
    } else if keyboard_input.just_pressed(KeyCode::Key2) {
        Some(2)
    } else if keyboard_input.just_pressed(KeyCode::Key3) {
        Some(3)
    } else if keyboard_input.just_pressed(KeyCode::Key4) {
        Some(4)
    } else if keyboard_input.just_pressed(KeyCode::Key5) {
        Some(5)
    } else if keyboard_input.just_pressed(KeyCode::Key6) {
        Some(6)
    } else if keyboard_input.just_pressed(KeyCode::Key7) {
        Some(7)
    } else if keyboard_input.just_pressed(KeyCode::Key8) {
        Some(8)
    } else if keyboard_input.just_pressed(KeyCode::Key9) {
        Some(9)
    } else {
        None
    }
}
// End Helper Functions
