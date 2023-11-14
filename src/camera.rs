use bevy::prelude::*;

use crate::constants::*;
use crate::game::events::CameraZoomEvent;
use crate::game::{events::CameraMovementEvent, resources::GameState};

pub struct CameraMovementPlugin;

impl Plugin for CameraMovementPlugin {
    fn build(&self, app: &mut App) {
        let compound_state = || {
            in_state(GameState::Exploring)
                .or_else(in_state(GameState::NonPlayerTurns))
                .or_else(in_state(GameState::EditingMap))
        };
        app.add_systems(Update, move_camera_system.run_if(compound_state()))
            .add_systems(Update, update_camera_zoom.run_if(compound_state()));
    }
}

// Systems
fn update_camera_zoom(
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut zoom_event_reader: EventReader<CameraZoomEvent>,
) {
    // TODO Add bounds to the zoom
    for mut orthographic_projection in query.iter_mut() {
        for event in zoom_event_reader.iter() {
            let amount = -CAMERA_ZOOM_SPEED * (event.0 as f32);
            let log_zoom = orthographic_projection.scale
                * CAMERA_ZOOM_LOG_BASE.powf(CAMERA_ZOOM_SPEED * amount);
            orthographic_projection.scale = log_zoom;
        }
    }
}

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
