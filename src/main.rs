mod assets;
mod camera;
mod constants;
mod game;
mod input;
mod map_editor;
mod menu;
mod specs;
mod ui;

use bevy::prelude::*;
use bevy_mod_raycast::DefaultRaycastingPlugin;

use assets::AssetsPlugin;
use game::events::EventsPlugin;
use game::exploring::ExploringPlugin;
use game::interacting::InteractingPlugin;
use game::pause::PausePlugin;
use game::world::WorldPlugin;
use input::InputPlugin;
use menu::MenuPlugin;

use crate::camera::CameraMovementPlugin;
use crate::game::resources::*;
use crate::input::MouseoverRaycastSet;
use crate::map_editor::map_editor::MapEditorPlugin;
use crate::ui::UIPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DefaultRaycastingPlugin::<MouseoverRaycastSet>::default(),
        ))
        .add_plugins(AssetsPlugin)
        .add_plugins(CameraMovementPlugin)
        .add_plugins(EventsPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(PausePlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(ExploringPlugin)
        .add_plugins(InteractingPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(MapEditorPlugin)
        .add_state::<GameState>()
        .run();
}
