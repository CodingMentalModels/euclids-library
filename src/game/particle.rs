use bevy::prelude::*;
use bevy::utils::Duration;
use rand::distributions::{Distribution, WeightedIndex};
use rand::prelude::*;
use rand::Rng;
use rand_distr::Exp;
use serde::{Deserialize, Serialize};

use super::{
    events::Direction,
    map::TileLocation,
    ui_state::{AsciiTileAppearance, TileAppearance, TileGrid},
};

// Components

#[derive(Component, Clone)]
pub struct ParticleEmitterComponent {
    pub spec: ParticleSpec,
    pub timer: Timer,
}

impl ParticleEmitterComponent {
    pub fn new(spec: ParticleSpec, timer: Timer) -> Self {
        Self { spec, timer }
    }

    pub fn emit(&self, commands: &mut Commands, font: Handle<Font>, origin: Vec2) -> Entity {
        let movement = self.spec.movement.clone();
        let appearance = self.spec.appearance.clone();
        let timer = movement.get_timer();

        let initial_appearance = appearance.get_appearance();
        let location = self.spec.emission_location.get_relative_location();

        let position = origin + TileGrid::tile_to_world_coordinates(location);
        let entity = initial_appearance.render(&mut commands.spawn_empty(), font.clone(), position);
        info!("Emmitted to {}", position);

        commands
            .entity(entity)
            .insert(ParticleComponent::new(movement, appearance, timer))
            .id()
    }
}

#[derive(Component, Clone)]
pub struct ParticleComponent {
    pub particle_movement: ParticleMovement,
    pub particle_appearance: ParticleAppearance,
    pub timer: Timer,
}

impl ParticleComponent {
    pub fn new(
        particle_movement: ParticleMovement,
        particle_appearance: ParticleAppearance,
        timer: Timer,
    ) -> Self {
        Self {
            particle_movement,
            particle_appearance,
            timer,
        }
    }
}

// End Components

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticleSpec {
    pub emission_timing: ParticleTiming,
    pub emission_location: ParticleLocation,
    pub movement: ParticleMovement,
    pub appearance: ParticleAppearance,
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
            .insert(ParticleEmitterComponent::new(
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
            Self::Once(d) => d.clone(),
            Self::Every(d) => d.clone(),
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

                let lambda = 1. / (duration.as_millis() as f32);
                info!("lambda: {}", lambda);
                let exp = Exp::new(lambda).unwrap();
                let v: f32 = exp.sample(&mut rng);
                info!("exponential: {}", v);
                Timer::new(Duration::from_millis(v.round() as u64), TimerMode::Once)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleLocation {
    Exact(TileLocation),
}

impl ParticleLocation {
    pub fn get_relative_location(&self) -> TileLocation {
        match self {
            Self::Exact(location) => *location,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticleMovement {
    pub timing: ParticleTiming,
    pub direction: ParticleDirection,
}

impl ParticleMovement {
    pub fn new(timing: ParticleTiming, direction: ParticleDirection) -> Self {
        Self { timing, direction }
    }

    pub fn get_timer(&self) -> Timer {
        self.timing.get_timer()
    }

    pub fn get_translation(&self) -> Vec2 {
        self.direction.get_translation()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleDirection {
    Constant(Direction),
    Weighted(Vec<(Direction, u32)>),
}

impl ParticleDirection {
    pub fn get_translation(&self) -> Vec2 {
        let direction = match self {
            Self::Constant(direction) => *direction,
            Self::Weighted(directions_and_weights) => {
                let dist =
                    WeightedIndex::new(directions_and_weights.iter().map(|(_, w)| *w)).unwrap();
                let dir_idx = rand::thread_rng().sample(&dist);
                directions_and_weights[dir_idx].0.clone()
            }
        };
        TileGrid::tile_to_world_coordinates(direction.as_tile_location())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleAppearance {
    Constant(AsciiTileAppearance),
}

impl ParticleAppearance {
    pub fn get_appearance(&self) -> TileAppearance {
        match self {
            Self::Constant(appearance) => TileAppearance::Ascii(appearance.clone()),
        }
    }
}
