use bevy::prelude::*;

use super::{
    constants::*,
    events::{CameraMovementEvent, MovementEvent, StateChangeEvent},
    map::{MapLayer, SurfaceTile, Tile},
    particle::{ParticleComponent, ParticleEmitterComponent, ParticleTiming},
    player::{LocationComponent, PlayerComponent},
    resources::{GameState, LoadedFont, LoadedMap},
};

pub struct ExploringPlugin;

impl Plugin for ExploringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LoadingMap), load_map_system)
            .add_systems(
                Update,
                move_player_system.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(
                Update,
                move_camera_system.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(
                Update,
                emit_particles_system.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(
                Update,
                update_particles_system.run_if(in_state(GameState::Exploring)),
            )
            .add_systems(
                Update,
                despawn_particles_offscreen_system.run_if(in_state(GameState::Exploring)),
            );
    }
}

// Systems

fn load_map_system(mut commands: Commands) {
    let mut map_layer = MapLayer::fill(
        DEFAULT_MAP_WIDTH_IN_TILES,
        DEFAULT_MAP_HEIGHT_IN_TILES,
        Tile::empty_ground(),
    );
    map_layer.update_edges(&Tile::wall());
    map_layer
        .update(5, 5, Tile::empty(SurfaceTile::Fireplace))
        .expect("5, 5 exists because we're setting it up that way.");

    commands.insert_resource(LoadedMap(map_layer.into()));
    commands.insert_resource(NextState(Some(GameState::Exploring)));
}

fn move_player_system(
    mut movement_event_reader: EventReader<MovementEvent>,
    mut player_query: Query<&mut LocationComponent, With<PlayerComponent>>,
) {
    let mut player_location = player_query.single_mut();

    for movement_event in movement_event_reader.iter() {
        player_location.translate(movement_event.0.as_tile_location());
    }
}

fn move_camera_system(
    mut camera_movement_event_reader: EventReader<CameraMovementEvent>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let mut transform = query.single_mut();
    for event in camera_movement_event_reader.iter() {
        transform.translation += event.0.as_vector().extend(0.) * CAMERA_MOVE_SPEED;
        info!("New transform: {:?}", transform.translation);
    }
}

fn emit_particles_system(
    mut commands: Commands,
    mut emitter_query: Query<(&mut ParticleEmitterComponent, &Transform)>,
    time: Res<Time>,
    font: Res<LoadedFont>,
) {
    for (mut emitter, transform) in emitter_query.iter_mut() {
        emitter.timer.tick(time.delta());

        if emitter.timer.just_finished() {
            emitter.emit(
                &mut commands,
                font.0.clone(),
                transform.translation.truncate(),
            );
            match &emitter.spec.emission_timing {
                ParticleTiming::Once(_) => { // Do nothing
                }
                ParticleTiming::Every(duration) => {
                    emitter.timer = duration.get_timer();
                }
            }
        }
    }
}

fn update_particles_system(
    mut particles_query: Query<(&mut ParticleComponent, &mut Transform)>,
    time: Res<Time>,
    font: Res<LoadedFont>,
) {
    for (mut particle, mut transform) in particles_query.iter_mut() {
        particle.timer.tick(time.delta());
        if particle.timer.just_finished() {
            match &particle.particle_movement.timing {
                ParticleTiming::Once(_) => {
                    // Do nothing
                }
                ParticleTiming::Every(duration) => {
                    particle.timer = duration.get_timer();
                }
            }

            transform.translation += particle.particle_movement.get_translation().extend(0.);

            // TODO Handle Appearance
        }
    }
}

fn despawn_particles_offscreen_system(
    mut commands: Commands,
    camera_query: Query<(&Transform, &OrthographicProjection)>,
    particle_query: Query<(Entity, &Transform), With<ParticleComponent>>,
    window_query: Query<&Window>,
) {
    // Get the main window (screen) dimensions.
    let window = window_query.single();
    let window_width = window.width() / 2.0;
    let window_height = window.height() / 2.0;

    // Assumes there's only one camera
    let (camera_transform, projection) = camera_query.single();

    // Get the scale of the camera.
    let camera_scale = projection.scale;

    // Adjust window dimensions based on camera scale.
    let adjusted_width = window_width * camera_scale;
    let adjusted_height = window_height * camera_scale;

    for (entity, transform) in particle_query.iter() {
        // Calculate the entity's position relative to the camera.
        let position_relative = transform.translation - camera_transform.translation;

        // Check if the entity is offscreen, accounting for the camera's scale.
        if position_relative.x < -adjusted_width
            || position_relative.x > adjusted_width
            || position_relative.y < -adjusted_height
            || position_relative.y > adjusted_height
        {
            info!("Despawning particle");
            commands.entity(entity).despawn_recursive();
        }
    }
}

// End Systems

// Components

// End Components

// Helper Functions

// End Helper Functions
