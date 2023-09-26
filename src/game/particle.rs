use bevy::prelude::*;
use bevy::utils::Duration;
use rand::prelude::*;
use rand_distr::{Distribution, Exp};
use serde::{Deserialize, Serialize};

use super::{
    events::Direction,
    map::TileLocation,
    ui_state::{AsciiTileAppearance, TileAppearance, TileGrid},
};

// Components

#[derive(Component, Clone)]
pub struct ParticleEmitterComponent(ParticleSpec, Timer);

#[derive(Component, Clone)]
pub struct ParticleComponent(ParticleMovement, ParticleAppearance, Timer);

// End Components

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
        commands
            .entity(entity)
            .insert(ParticleEmitterComponent(
                self.clone(),
                self.emission_timing.get_timer(),
            ))
            .id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleTiming {
    Once(ParticleDuration),
    Every(ParticleDuration),
}

impl ParticleTiming {
    pub fn get_timer(&self) -> Timer {
        self.get_duration().get_timer()
    }

    fn get_duration(&self) -> ParticleDuration {
        match self {
            Self::Once(d) => *d,
            Self::Every(d) => *d,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleDuration {
    Exact(Duration),
    Exponential(Duration),
}

impl ParticleDuration {
    pub fn get_timer(&self) -> Timer {
        match self {
            Self::Exact(duration) => Timer::new(*duration, TimerMode::Once),
            Self::Exponential(duration) => {
                let mut rng = thread_rng();

                let lambda = duration.as_nanos() as f32;
                let exp = Exp::new(lambda).unwrap();
                let v: f32 = exp.sample(&mut rng);
                Timer::new(Duration::new(0, v), TimerMode::Once)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleLocation {
    Exact(TileLocation),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticleMovement {
    timing: ParticleTiming,
    direction: ParticleDirection,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleDirection {
    Constant(Direction),
    Weighted(Vec<(Direction, u32)>),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleAppearance {
    Constant(AsciiTileAppearance),
}
