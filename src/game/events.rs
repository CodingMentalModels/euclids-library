use bevy::prelude::*;

use super::map::TileLocation;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Event)]
pub struct CameraZoomEvent(pub i32);

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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
