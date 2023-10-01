use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{map::TileLocation, resources::GameState};

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            change_state_system.run_if(on_event::<StateChangeEvent>()),
        );
    }
}

// Systems

fn change_state_system(
    mut commands: Commands,
    mut state_change_event_reader: EventReader<StateChangeEvent>,
) {
    for _event in state_change_event_reader.iter() {
        commands.insert_resource(NextState(Some(GameState::Interacting)));
    }
}

// End Systems

// Events

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Event)]
pub struct CameraZoomEvent(pub i32);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Event)]
pub struct CameraMovementEvent(pub Direction);

impl CameraMovementEvent {
    pub fn as_vector(&self) -> Vec2 {
        self.0.as_vector()
    }

    pub fn as_tile_location(&self) -> TileLocation {
        self.0.as_tile_location()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Event)]
pub struct MovementEvent(pub Direction);

impl MovementEvent {
    pub fn as_vector(&self) -> Vec2 {
        self.0.as_vector()
    }

    pub fn as_tile_location(&self) -> TileLocation {
        self.0.as_tile_location()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Event)]
pub struct DirectionEvent(pub Direction);

impl DirectionEvent {
    pub fn as_vector(&self) -> Vec2 {
        self.0.as_vector()
    }

    pub fn as_tile_location(&self) -> TileLocation {
        self.0.as_tile_location()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Event)]
pub struct StateChangeEvent(pub GameState);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl Direction {
    pub fn as_vector(&self) -> Vec2 {
        match self {
            Self::Up => Vec2::Y,
            Self::UpRight => Vec2::X + Vec2::Y,
            Self::Right => Vec2::X,
            Self::DownRight => Vec2::X + Vec2::NEG_Y,
            Self::Down => Vec2::NEG_Y,
            Self::DownLeft => Vec2::NEG_X + Vec2::NEG_Y,
            Self::Left => Vec2::NEG_X,
            Self::UpLeft => Vec2::NEG_X + Vec2::Y,
        }
    }

    pub fn as_tile_location(&self) -> TileLocation {
        match self {
            Self::Up => TileLocation::new(0, 1),
            Self::UpRight => TileLocation::new(1, 1),
            Self::Right => TileLocation::new(1, 0),
            Self::DownRight => TileLocation::new(1, -1),
            Self::Down => TileLocation::new(0, -1),
            Self::DownLeft => TileLocation::new(-1, -1),
            Self::Left => TileLocation::new(-1, 0),
            Self::UpLeft => TileLocation::new(-1, 1),
        }
    }
}

// End Events
