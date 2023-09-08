mod game;

use bevy::prelude::*;
use bevy_mod_raycast::DefaultRaycastingPlugin;

use game::assets::AssetsPlugin;
use game::exploring::ExploringPlugin;
use game::input::InputPlugin;
use game::pause::PausePlugin;

use crate::game::input::MouseoverRaycastSet;
use crate::game::resources::*;
use crate::game::ui::UIPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DefaultRaycastingPlugin::<MouseoverRaycastSet>::default(),
        ))
        .add_plugins(AssetsPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(PausePlugin)
        .add_plugins(ExploringPlugin)
        .add_state::<GameState>()
        .run();
}
