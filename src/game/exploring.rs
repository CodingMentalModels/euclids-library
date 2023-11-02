use std::collections::HashSet;

use bevy::prelude::*;

use super::character::{ActionClockComponent, BodyComponent, LocationComponent};
use super::enemy::{AIComponent, EnemyComponent};
use super::events::DamageEvent;
use super::resources::RngResource;
use super::{
    events::{CameraMovementEvent, TryMoveEvent},
    map::{MapLayer, SurfaceTile, Tile},
    npc::NPCComponent,
    particle::{ParticleComponent, ParticleEmitterComponent, ParticleTiming},
    player::PlayerComponent,
    resources::{GameState, LoadedFont, LoadedMap},
};
use crate::constants::*;
use crate::game::map::{AsciiTileAppearance, TileAppearance, TileGrid};
use crate::ui::LogState;

pub struct ExploringPlugin;

impl Plugin for ExploringPlugin {
    fn build(&self, app: &mut App) {
        let generalized_exploring =
            || in_state(GameState::Exploring).or_else(in_state(GameState::NonPlayerTurns));
        app.add_event::<DespawnNonCameraEntitiesEvent>()
            .add_systems(OnEnter(GameState::LoadingMap), load_map_system)
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
            .add_systems(
                OnEnter(GameState::NonPlayerTurns),
                determine_turn_order_system,
            )
            .add_systems(
                Update,
                process_non_player_turn.run_if(in_state(GameState::NonPlayerTurns)),
            )
            .add_systems(
                Update,
                despawn_non_camera_entities.run_if(on_event::<DespawnNonCameraEntitiesEvent>()),
            );
    }
}

//Events
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Event)]
pub struct DespawnNonCameraEntitiesEvent;

// End Events

// Resources

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Resource)]
pub struct ExploringUIState {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub struct ShouldSpawnMap(pub bool);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct NonPlayerTurnOrder(pub Vec<Entity>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct NonPlayerTurnLength(pub u8);

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
                                end_turn(&mut commands, MOVEMENT_TICKS);
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
    character_query: Query<
        (
            Entity,
            &LocationComponent,
            Option<&PlayerComponent>,
            Option<&NPCComponent>,
            Option<&EnemyComponent>,
        ),
        Or<(
            With<NPCComponent>,
            With<PlayerComponent>,
            With<EnemyComponent>,
        )>,
    >,
    enemy_query: Query<(Entity, &LocationComponent), With<EnemyComponent>>,
    map: Res<LoadedMap>,
    font: Res<LoadedFont>,
    mut should_spawn: ResMut<ShouldSpawnMap>,
) {
    if !should_spawn.0 {
        return;
    }

    let (player_entity, player_location, _, _, _) = character_query
        .iter()
        .find(|(entity, _location, maybe_player_component, _, _)| maybe_player_component.is_some())
        .expect("The player must exist.");

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
    let npc_tile = TileAppearance::Ascii('&'.into());
    let enemy_tile = TileAppearance::Ascii('s'.into());

    for (entity, location, maybe_player, maybe_npc, maybe_enemy) in character_query.iter() {
        let maybe_player_tile = maybe_player.map(|_| player_tile.clone());
        let maybe_npc_tile = maybe_npc.map(|_| npc_tile.clone());
        let maybe_enemy_tile = maybe_enemy.map(|_| enemy_tile.clone());

        let tile_appearance = maybe_player_tile
            .or(maybe_npc_tile)
            .or(maybe_enemy_tile)
            .expect("Must be one of the 3 types given the query.");

        let sprite = tile_appearance.render(
            &mut commands
                .get_entity(entity)
                .expect("Entity must exist if it was returned from the query."),
            font.0.clone(),
            TileGrid::tile_to_world_coordinates(location.0.get_tile_location()),
        );

        if maybe_player.is_some() {
            commands.entity(sprite).insert(PlayerSprite);
        }
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

fn determine_turn_order_system(
    mut commands: Commands,
    non_player_query: Query<(Entity, &ActionClockComponent)>,
) {
    let mut results: Vec<(Entity, &ActionClockComponent)> =
        non_player_query.iter().collect::<Vec<_>>();

    // Should be ascending because we pop off of NonPlayerTurnOrder
    results.sort_by(|a, b| a.1.get_remaining().cmp(&b.1.get_remaining()));

    commands.insert_resource(NonPlayerTurnOrder(
        results.iter().map(|(e, _)| e).cloned().collect(),
    ));
}

fn process_non_player_turn(
    mut commands: Commands,
    mut non_player_turns: ResMut<NonPlayerTurnOrder>,
    mut non_player_query: Query<(Entity, &mut ActionClockComponent, &mut AIComponent)>,
    mut log: ResMut<LogState>,
    non_player_turn_length: Res<NonPlayerTurnLength>,
) {
    match non_player_turns.0.pop() {
        Some(non_player_entity) => {
            for (acting_entity, mut action_clock, mut ai) in non_player_query.iter_mut() {
                if acting_entity == non_player_entity {
                    if action_clock.tick_and_is_finished(non_player_turn_length.0) {
                        log.log_string(&format!("{:?} takes its turn", non_player_entity));
                        action_clock.reset(ai.next().get_ticks());
                    } else {
                        log.log_string(&format!(
                            "{:?} didn't get to take its turn",
                            non_player_entity
                        ));
                    }
                }
            }
        }
        None => {
            commands.insert_resource(NextState(Some(GameState::Exploring)));
        }
    }
}

fn despawn_non_camera_entities(
    mut commands: Commands,
    query: Query<(Entity, &Transform), Without<Camera>>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }
}

// End Systems

// Components
#[derive(Component, Clone, Copy)]
struct PlayerSprite;

// End Components

// Helper Functions
fn end_turn(mut commands: &mut Commands, n_ticks: u8) {
    commands.insert_resource(NonPlayerTurnLength(n_ticks));
    commands.insert_resource(NextState(Some(GameState::NonPlayerTurns)));
}
// End Helper Functions
