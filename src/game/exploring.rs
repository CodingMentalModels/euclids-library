use std::collections::HashSet;

use bevy::prelude::*;

use super::character::{BodyComponent, LocationComponent};
use super::events::DamageEvent;
use super::resources::RngResource;
use super::ui_state::LogState;
use super::{
    constants::*,
    events::{CameraMovementEvent, TryMoveEvent},
    map::{MapLayer, SurfaceTile, Tile},
    npc::NPCComponent,
    particle::{ParticleComponent, ParticleEmitterComponent, ParticleTiming},
    player::PlayerComponent,
    resources::{GameState, LoadedFont, LoadedMap},
    ui_state::{AsciiTileAppearance, TileAppearance, TileGrid},
};

pub struct ExploringPlugin;

impl Plugin for ExploringPlugin {
    fn build(&self, app: &mut App) {
        let generalized_exploring =
            || in_state(GameState::Exploring).or_else(in_state(GameState::NPCTurns));
        app.add_systems(OnEnter(GameState::LoadingMap), load_map_system)
            .add_systems(OnEnter(GameState::Exploring), spawn_map)
            .add_systems(Update, movement_system.run_if(generalized_exploring()))
            .add_systems(Update, update_positions.run_if(generalized_exploring()))
            .add_systems(Update, move_camera_system.run_if(generalized_exploring()))
            .add_systems(Update, handle_damage_system.run_if(generalized_exploring()))
            .add_systems(
                Update,
                emit_particles_system.run_if(generalized_exploring()),
            )
            .add_systems(
                Update,
                update_particles_system.run_if(generalized_exploring()),
            )
            .add_systems(
                Update,
                despawn_particles_offscreen_system.run_if(generalized_exploring()),
            )
            .add_systems(OnEnter(GameState::NPCTurns), determine_turn_order_system)
            .add_systems(
                Update,
                process_npc_turn.run_if(in_state(GameState::NPCTurns)),
            );
    }
}

// Resources

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub struct ShouldSpawnMap(pub bool);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct NPCTurnOrder(pub Vec<Entity>);

// End Resources

// Events

// End Events

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
    commands.insert_resource(ShouldSpawnMap(true));
}

fn movement_system(
    mut commands: Commands,
    mut movement_event_reader: EventReader<TryMoveEvent>,
    mut query: Query<(Entity, &mut LocationComponent, Option<&PlayerComponent>)>,
    mut log: ResMut<LogState>,
    map: Res<LoadedMap>,
) {
    let non_traversable_entity_locations = query
        .iter()
        .map(|(_, location, _)| location.0)
        .collect::<HashSet<_>>();
    for try_move_event in movement_event_reader.iter() {
        for (entity, mut location, _maybe_is_player) in query.iter_mut() {
            let TryMoveEvent(entity_to_move, direction) = try_move_event;
            if entity == *entity_to_move {
                let final_location = location.translated(direction.as_tile_location());
                if non_traversable_entity_locations.contains(&final_location.0) {
                    log.log_string("Trying to walk into another entity.");
                } else {
                    match map.0.is_traversable(final_location.0) {
                        Err(_e) => {
                            log.log_string("Trying to walk off the map.");
                        }
                        Ok(is_traversable) => {
                            if is_traversable {
                                *location = final_location;
                                commands.insert_resource(NextState(Some(GameState::NPCTurns)));
                            } else {
                                log.log_string("Trying to traverse non-traversable terrain.");
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_positions(mut query: Query<(&LocationComponent, &mut Transform)>) {
    for (location, mut transform) in query.iter_mut() {
        let screen_coordinates =
            TileGrid::tile_to_world_coordinates(location.0.get_tile_location());
        *transform = Transform::from_translation(screen_coordinates.extend(0.));
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

fn spawn_map(
    mut commands: Commands,
    player_query: Query<(Entity, &LocationComponent), With<PlayerComponent>>,
    npc_query: Query<(Entity, &LocationComponent), With<NPCComponent>>,
    map: Res<LoadedMap>,
    font: Res<LoadedFont>,
    mut should_spawn: ResMut<ShouldSpawnMap>,
) {
    if !should_spawn.0 {
        info!("Ignoring spawn_map.");
        return;
    }

    info!("Spawning map.");
    let (player_entity, player_location) = player_query.single();

    let map_layer = map
        .0
        .get_layer(player_location.0.get_map_layer())
        .expect("Player's layer must exist.");

    let tile_grid = TileGrid::from_map_layer(map_layer.clone());
    tile_grid.render(&mut commands, font.0.clone());

    map_layer
        .as_location_and_tile_vector()
        .into_iter()
        .for_each(|(location, tile)| {
            if let Some(particle_spec) = tile.get_surface().get_particle_spec() {
                particle_spec.render(&mut commands, font.0.clone(), location);
            };
        });

    let player_tile = TileAppearance::Ascii(AsciiTileAppearance::from('@'));
    let player_sprite = player_tile.render(
        &mut commands
            .get_entity(player_entity)
            .expect("Player entity must exist if it was returned from the query."),
        font.0.clone(),
        TileGrid::tile_to_world_coordinates(player_location.0.get_tile_location()),
    );
    commands.entity(player_sprite).insert(PlayerSprite);

    for (entity, location) in npc_query.iter() {
        let npc_tile = TileAppearance::Ascii('&'.into());
        let _npc_sprite = npc_tile.render(
            &mut commands
                .get_entity(entity)
                .expect("NPC Entity must exist if it was returned from the query."),
            font.0.clone(),
            TileGrid::tile_to_world_coordinates(location.0.get_tile_location()),
        );
    }

    should_spawn.0 = false;
}

fn handle_damage_system(
    mut character_query: Query<(Entity, &mut BodyComponent)>,
    mut damage_event_reader: EventReader<DamageEvent>,
    mut rng: ResMut<RngResource>,
) {
    for damage_event in damage_event_reader.iter() {
        let DamageEvent(damaged_entity, damage) = damage_event;
        for (entity, mut body_component) in character_query.iter_mut() {
            if *damaged_entity == entity {
                body_component.0.take_damage(&mut *rng, damage.clone());
            }
        }
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
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn determine_turn_order_system(mut commands: Commands, query: Query<Entity, With<NPCComponent>>) {
    // TODO: Do this based on how many auts of overflow they had
    commands.insert_resource(NPCTurnOrder(query.iter().collect::<Vec<Entity>>()));
}

fn process_npc_turn(
    mut commands: Commands,
    mut npc_turns: ResMut<NPCTurnOrder>,
    mut log: ResMut<LogState>,
) {
    match npc_turns.0.pop() {
        Some(npc) => {
            log.log_string(&format!("NPC {:?} takes its turn", npc));
        }
        None => {
            commands.insert_resource(NextState(Some(GameState::Exploring)));
        }
    }
}

// End Systems

// Components
#[derive(Component, Clone, Copy)]
struct PlayerSprite;

// End Components

// Helper Functions

// End Helper Functions
