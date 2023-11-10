use bevy::prelude::*;

use crate::constants::*;
use crate::game::{events::CameraMovementEvent, resources::GameState};

pub struct CameraMovementPlugin;

impl Plugin for CameraMovementPlugin {
    fn build(&self, app: &mut App) {
        let compound_state = || {
            in_state(GameState::Exploring)
                .or_else(in_state(GameState::NonPlayerTurns))
                .or_else(in_state(GameState::EditingMap))
        };
        app.add_systems(Update, move_camera_system.run_if(compound_state()));
    }
}

// Systems

fn move_camera_system(
    mut camera_movement_event_reader: EventReader<CameraMovementEvent>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let mut transform = query.single_mut();
    for event in camera_movement_event_reader.iter() {
        transform.translation += event.0.as_vector().extend(0.) * CAMERA_MOVE_SPEED;
    }
}

// End Systems
