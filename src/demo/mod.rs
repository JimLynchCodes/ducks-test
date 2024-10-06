//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

mod animation;
pub mod level;
mod movement;
pub mod player;
pub mod other_player;
pub mod background;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        background::plugin,
        animation::plugin,
        movement::plugin,
        player::plugin,
        level::plugin,
        other_player::plugin
    ));
}
