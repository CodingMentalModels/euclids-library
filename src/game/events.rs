use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Event)]
pub struct MovementEvent(pub Direction);

impl MovementEvent {
    pub fn as_vector(&self) -> Vec2 {
        self.0.as_vector()
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
}
