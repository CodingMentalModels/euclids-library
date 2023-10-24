use std::collections::HashSet;
use std::time::Duration;
use std::unreachable;

use bevy::prelude::*;
use bevy_mod_raycast::{
    print_intersections, DefaultRaycastingPlugin, RaycastMethod, RaycastSource, RaycastSystem,
};

use super::constants::*;
use super::events::DamageEvent;
use super::events::{
    CameraMovementEvent, CameraZoomEvent, ChooseDirectionEvent, Direction, OpenMenuEvent,
    ProgressPromptEvent, StateChangeEvent, TryMoveEvent,
};
use super::menu::MenuType;
use super::player::PlayerComponent;
use super::resources::GameState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PauseUnpauseEvent>()
            .add_event::<OpenMenuEvent>()
            .add_event::<CameraMovementEvent>()
            .add_event::<CameraZoomEvent>()
            .add_event::<TryMoveEvent>()
            .add_event::<DamageEvent>()
            .add_event::<StateChangeEvent>()
            .add_event::<ChooseDirectionEvent>()
            .add_event::<ProgressPromptEvent>()
            .insert_resource(KeyHoldTimer::default())
            .add_systems(
                First,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays::<MouseoverRaycastSet>),
            )
            //.add_systems(Update, print_intersections::<MouseoverRaycastSet>)
            .add_systems(Update, input_system);
    }
}

// Components

// End Components

// Resources

#[derive(Debug, Resource)]
pub struct KeyHoldTimer {
    timer: Timer,
    key: Option<KeyCode>,
}

impl Default for KeyHoldTimer {
    fn default() -> Self {
        KeyHoldTimer {
            timer: Timer::new(
                Duration::from_millis(KEY_HOLD_DELAY_IN_MILLIS),
                TimerMode::Once,
            ),
            key: None,
        }
    }
}

impl KeyHoldTimer {
    fn new(key: KeyCode) -> Self {
        let mut to_return = Self::default();
        to_return.set_key(key);
        to_return
    }

    fn finished(&self) -> bool {
        self.timer.finished()
    }

    fn get_key(&self) -> Option<KeyCode> {
        self.key
    }

    fn has_key(&self) -> bool {
        self.key.is_some()
    }

    fn set_key(&mut self, key: KeyCode) {
        self.key = Some(key);
    }

    fn reset(&mut self) {
        self.timer.reset();
        self.key = None;
    }

    fn reset_with(&mut self, key: KeyCode) {
        self.timer.reset();
        self.key = Some(key);
    }

    fn tick_and_should_trigger(
        &mut self,
        delta: Duration,
        keyboard_input: &Res<Input<KeyCode>>,
    ) -> bool {
        self.timer.tick(delta);
        let mut pressed_keys = keyboard_input
            .get_pressed()
            .filter(|key| !is_modifier_key(**key))
            .collect::<Vec<_>>();
        if pressed_keys.len() != 1 {
            // Multiple keys pressed.  Resetting.
            self.reset();
            return true;
        }

        let key_code = *(pressed_keys
            .pop()
            .expect("We just checked that the length is 1."));
        match self.key {
            None => {
                // No previous key set
                self.reset_with(key_code);
                true
            }
            Some(held_key) => {
                if held_key != key_code {
                    // New key doesn't match old key.  Resetting with the new key.
                    self.reset_with(key_code);
                    true
                } else {
                    // Check the timer to see whether we should trigger
                    if self.finished() {
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }
}

// End Resources

// Events

#[derive(Debug, Clone, Copy, PartialEq, Eq, Event)]
pub struct PauseUnpauseEvent;

// End Events

// Systems
pub fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    state: Res<State<GameState>>,
    mut timer: ResMut<KeyHoldTimer>,
    time: Res<Time>,
    mut pause_unpause_event_writer: EventWriter<PauseUnpauseEvent>,
    state_change_event_writer: EventWriter<StateChangeEvent>,
    camera_movement_event_writer: EventWriter<CameraMovementEvent>,
    zoom_event_writer: EventWriter<CameraZoomEvent>,
    open_menu_event_writer: EventWriter<OpenMenuEvent>,
    movement_event_writer: EventWriter<TryMoveEvent>,
    choose_direction_event_writer: EventWriter<ChooseDirectionEvent>,
    progress_prompt_event_writer: EventWriter<ProgressPromptEvent>,
    player_entity_query: Query<Entity, With<PlayerComponent>>,
) {
    if keyboard_input.get_just_released().count() > 0 {
        timer.reset();
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        pause_unpause_event_writer.send(PauseUnpauseEvent);
    }

    match state.get() {
        GameState::Exploring => {
            handle_camera_movement(
                &keyboard_input,
                &mut timer,
                time.delta(),
                camera_movement_event_writer,
            );
            handle_camera_zoom(&keyboard_input, &mut timer, time.delta(), zoom_event_writer);
            handle_movement(
                &keyboard_input,
                &mut timer,
                time.delta(),
                movement_event_writer,
                player_entity_query,
            );
            handle_interact(&keyboard_input, state_change_event_writer);
            handle_open_menu(&keyboard_input, open_menu_event_writer);
        }
        GameState::Interacting => {
            handle_progress_prompt(&keyboard_input, progress_prompt_event_writer);
            handle_exit(
                &keyboard_input,
                state_change_event_writer,
                GameState::Exploring,
            );
            handle_choose_direction(&keyboard_input, choose_direction_event_writer);
        }
        GameState::Menu => {
            handle_exit(
                &keyboard_input,
                state_change_event_writer,
                GameState::Exploring,
            );
        }
        _ => {}
    }
}

// Update our `RaycastSource` with the current cursor position every frame.
fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<MouseoverRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let Some(cursor_moved) = cursor.iter().last() else {
        return;
    };
    for mut pick_source in &mut query {
        pick_source.cast_method = RaycastMethod::Screenspace(cursor_moved.position);
    }
}
// End Systems

// Structs
#[derive(Debug, Clone, Reflect)]
pub struct MouseoverRaycastSet;

// End Structs

// Helper Functions

fn handle_movement(
    keyboard_input: &Res<Input<KeyCode>>,
    timer: &mut KeyHoldTimer,
    delta: Duration,
    mut movement_event_writer: EventWriter<TryMoveEvent>,
    player_query: Query<Entity, With<PlayerComponent>>,
) {
    let player_entity = player_query
        .get_single()
        .expect("Handle movement should only be run once a player exists.");
    match get_direction_from_keycode(keyboard_input) {
        Some(direction) => {
            if timer.tick_and_should_trigger(delta, keyboard_input) {
                movement_event_writer.send(TryMoveEvent(player_entity, direction));
            }
        }
        _ => {
            // Do nothing
        }
    }
}

fn handle_camera_zoom(
    keyboard_input: &Res<Input<KeyCode>>,
    timer: &mut KeyHoldTimer,
    delta: Duration,
    mut zoom_event_writer: EventWriter<CameraZoomEvent>,
) {
    if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && keyboard_input.pressed(KeyCode::Equals)
    {
        if timer.tick_and_should_trigger(delta, keyboard_input) {
            zoom_event_writer.send(CameraZoomEvent(1));
        }
    }

    if keyboard_input.pressed(KeyCode::Minus) {
        if timer.tick_and_should_trigger(delta, keyboard_input) {
            zoom_event_writer.send(CameraZoomEvent(-1));
        }
    }
}

fn handle_camera_movement(
    keyboard_input: &Res<Input<KeyCode>>,
    mut timer: &mut KeyHoldTimer,
    delta: Duration,
    mut camera_movement_event_writer: EventWriter<CameraMovementEvent>,
) {
    if keyboard_input.pressed(KeyCode::W) {
        if timer.tick_and_should_trigger(delta, keyboard_input) {
            camera_movement_event_writer.send(CameraMovementEvent(Direction::Up));
        }
    }
    if keyboard_input.pressed(KeyCode::A) {
        if timer.tick_and_should_trigger(delta, keyboard_input) {
            camera_movement_event_writer.send(CameraMovementEvent(Direction::Left));
        }
    }
    if keyboard_input.pressed(KeyCode::S) {
        if timer.tick_and_should_trigger(delta, keyboard_input) {
            camera_movement_event_writer.send(CameraMovementEvent(Direction::Down));
        }
    }
    if keyboard_input.pressed(KeyCode::D) {
        if timer.tick_and_should_trigger(delta, keyboard_input) {
            camera_movement_event_writer.send(CameraMovementEvent(Direction::Right));
        }
    }
}

fn handle_interact(
    keyboard_input: &Res<Input<KeyCode>>,
    mut state_change_event_writer: EventWriter<StateChangeEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state_change_event_writer.send(StateChangeEvent(GameState::Interacting));
    }
}

fn handle_open_menu(
    keyboard_input: &Res<Input<KeyCode>>,
    mut open_menu_event_writer: EventWriter<OpenMenuEvent>,
) {
    if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && keyboard_input.just_pressed(KeyCode::Key2)
    {
        open_menu_event_writer.send(OpenMenuEvent(MenuType::Character));
    }
}

fn handle_exit(
    keyboard_input: &Res<Input<KeyCode>>,
    mut state_change_event_writer: EventWriter<StateChangeEvent>,
    state: GameState,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state_change_event_writer.send(StateChangeEvent(state));
    }
}

fn handle_progress_prompt(
    keyboard_input: &Res<Input<KeyCode>>,
    mut progress_prompt_event_writer: EventWriter<ProgressPromptEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::Return) {
        progress_prompt_event_writer.send(ProgressPromptEvent::Continue);
    } else if let Some(digit) = get_digit_from_keycode(keyboard_input) {
        progress_prompt_event_writer.send(ProgressPromptEvent::ChooseOption(digit));
    }
}

fn handle_choose_direction(
    keyboard_input: &Res<Input<KeyCode>>,
    mut choose_direction_event_writer: EventWriter<ChooseDirectionEvent>,
) {
    match get_direction_from_keycode(keyboard_input) {
        Some(direction) => {
            choose_direction_event_writer.send(ChooseDirectionEvent(direction));
        }
        _ => {
            // Do nothing
        }
    }
}

fn get_direction_from_keycode(keyboard_input: &Res<Input<KeyCode>>) -> Option<Direction> {
    if keyboard_input.any_pressed([KeyCode::Numpad8, KeyCode::K]) {
        Some(Direction::Up)
    } else if keyboard_input.any_pressed([KeyCode::Numpad2, KeyCode::J]) {
        Some(Direction::Down)
    } else if keyboard_input.any_pressed([KeyCode::Numpad4, KeyCode::H]) {
        Some(Direction::Left)
    } else if keyboard_input.any_pressed([KeyCode::Numpad6, KeyCode::L]) {
        Some(Direction::Right)
    } else if keyboard_input.any_pressed([KeyCode::Numpad7, KeyCode::Y]) {
        Some(Direction::UpLeft)
    } else if keyboard_input.any_pressed([KeyCode::Numpad9, KeyCode::U]) {
        Some(Direction::UpRight)
    } else if keyboard_input.any_pressed([KeyCode::Numpad1, KeyCode::B]) {
        Some(Direction::DownLeft)
    } else if keyboard_input.any_pressed([KeyCode::Numpad3, KeyCode::N]) {
        Some(Direction::DownRight)
    } else {
        None
    }
}

fn get_digit_from_keycode(keyboard_input: &Res<Input<KeyCode>>) -> Option<usize> {
    if keyboard_input.just_pressed(KeyCode::Numpad0) {
        Some(0)
    } else if keyboard_input.just_pressed(KeyCode::Numpad1) {
        Some(1)
    } else if keyboard_input.just_pressed(KeyCode::Numpad2) {
        Some(2)
    } else if keyboard_input.just_pressed(KeyCode::Numpad3) {
        Some(3)
    } else if keyboard_input.just_pressed(KeyCode::Numpad4) {
        Some(4)
    } else if keyboard_input.just_pressed(KeyCode::Numpad5) {
        Some(5)
    } else if keyboard_input.just_pressed(KeyCode::Numpad6) {
        Some(6)
    } else if keyboard_input.just_pressed(KeyCode::Numpad7) {
        Some(7)
    } else if keyboard_input.just_pressed(KeyCode::Numpad8) {
        Some(8)
    } else if keyboard_input.just_pressed(KeyCode::Numpad9) {
        Some(9)
    } else if keyboard_input.just_pressed(KeyCode::Key0) {
        Some(0)
    } else if keyboard_input.just_pressed(KeyCode::Key1) {
        Some(1)
    } else if keyboard_input.just_pressed(KeyCode::Key2) {
        Some(2)
    } else if keyboard_input.just_pressed(KeyCode::Key3) {
        Some(3)
    } else if keyboard_input.just_pressed(KeyCode::Key4) {
        Some(4)
    } else if keyboard_input.just_pressed(KeyCode::Key5) {
        Some(5)
    } else if keyboard_input.just_pressed(KeyCode::Key6) {
        Some(6)
    } else if keyboard_input.just_pressed(KeyCode::Key7) {
        Some(7)
    } else if keyboard_input.just_pressed(KeyCode::Key8) {
        Some(8)
    } else if keyboard_input.just_pressed(KeyCode::Key9) {
        Some(9)
    } else {
        None
    }
}

fn is_modifier_key(key: KeyCode) -> bool {
    match key {
        KeyCode::ShiftLeft
        | KeyCode::ShiftRight
        | KeyCode::ControlLeft
        | KeyCode::ControlRight
        | KeyCode::AltLeft
        | KeyCode::AltRight
        | KeyCode::SuperLeft
        | KeyCode::SuperRight => true,
        _ => false,
    }
}
// End Helper Functions
