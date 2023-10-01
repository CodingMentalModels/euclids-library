mod game;

use bevy::prelude::*;
use bevy_mod_raycast::DefaultRaycastingPlugin;

use game::assets::AssetsPlugin;
use game::events::EventsPlugin;
use game::exploring::ExploringPlugin;
use game::input::InputPlugin;
use game::interacting::InteractingPlugin;
use game::pause::PausePlugin;
use game::world::WorldPlugin;

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
        .add_plugins(EventsPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(PausePlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(ExploringPlugin)
        .add_plugins(InteractingPlugin)
        .add_state::<GameState>()
        .run();
}
