use bevy::prelude::*;
use bevy_mod_raycast::{
    print_intersections, DefaultRaycastingPlugin, RaycastMethod, RaycastSource, RaycastSystem,
};

use super::events::{CameraMovementEvent, Direction, MovementEvent};
use super::resources::GameState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PauseUnpauseEvent>()
            .add_event::<MovementEvent>()
            .add_event::<CameraMovementEvent>()
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
    mut movement_event_writer: EventWriter<MovementEvent>,
    mut camera_movement_event_writer: EventWriter<CameraMovementEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        pause_unpause_event_writer.send(PauseUnpauseEvent);
    }

    match state.get() {
        GameState::Exploring => {
            handle_movement(&keyboard_input, movement_event_writer);
            handle_camera_movement(&keyboard_input, camera_movement_event_writer);
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
    if keyboard_input.just_pressed(KeyCode::Numpad8) {
        movement_event_writer.send(MovementEvent(Direction::Up));
    }

    if keyboard_input.just_pressed(KeyCode::Numpad2) {
        movement_event_writer.send(MovementEvent(Direction::Down));
    }

    if keyboard_input.just_pressed(KeyCode::Numpad4) {
        movement_event_writer.send(MovementEvent(Direction::Left));
    }

    if keyboard_input.just_pressed(KeyCode::Numpad6) {
        movement_event_writer.send(MovementEvent(Direction::Right));
    }

    if keyboard_input.just_pressed(KeyCode::Numpad7) {
        movement_event_writer.send(MovementEvent(Direction::UpLeft));
    }

    if keyboard_input.just_pressed(KeyCode::Numpad9) {
        movement_event_writer.send(MovementEvent(Direction::UpRight));
    }

    if keyboard_input.just_pressed(KeyCode::Numpad1) {
        movement_event_writer.send(MovementEvent(Direction::DownLeft));
    }

    if keyboard_input.just_pressed(KeyCode::Numpad3) {
        movement_event_writer.send(MovementEvent(Direction::DownRight));
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

// End Helper Functions
