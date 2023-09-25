use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    events::Direction,
    map::TileLocation,
    ui_state::{AsciiTileAppearance, TileAppearance, TileGrid},
};

type Milliseconds = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticleSpec {
    emission_timing: ParticleTiming,
    emission_location: ParticleLocation,
    movement: ParticleMovement,
    appearance: ParticleAppearance,
}

impl ParticleSpec {
    pub fn new(
        emission_timing: ParticleTiming,
        emission_location: ParticleLocation,
        movement: ParticleMovement,
        appearance: ParticleAppearance,
    ) -> Self {
        Self {
            emission_timing,
            emission_location,
            movement,
            appearance,
        }
    }

    pub fn render(
        &self,
        commands: &mut Commands,
        font: Handle<Font>,
        location: TileLocation,
    ) -> Entity {
        let initial_appearance = match &self.appearance {
            ParticleAppearance::Constant(appearance) => appearance.clone(),
        };
        let entity = TileAppearance::Ascii(initial_appearance).render(
            &mut commands.spawn_empty(),
            font.clone(),
            TileGrid::tile_to_world_coordinates(location),
        );
        entity
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleTiming {
    Once(ParticleDuration),
    Every(ParticleDuration),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleDuration {
    Exact(Milliseconds),
    Exponential(Milliseconds),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleLocation {
    Exact(TileLocation),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleMovement {
    Constant(Direction),
    Weighted(Vec<(Direction, u32)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleAppearance {
    Constant(AsciiTileAppearance),
}
