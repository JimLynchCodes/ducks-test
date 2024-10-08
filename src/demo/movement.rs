//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;

use crate::AppSet;

use super::{websocket_join_msg::JoinRequestEvent, websocket_move_msg::MoveRequestEvent};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();

    app.add_systems(Update, apply_movement.chain().in_set(AppSet::Update));
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics
    /// engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 400.0,
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    // mut movement_query: Query<(&MovementController, &mut Transform)>,
    // mut cameras: Query<&mut Transform, With<Camera>>,
    mut param_set: ParamSet<(
        Query<(&MovementController, &mut Transform)>,
        Query<&mut Transform, With<Camera>>,
    )>,
    mut move_request_event_writer: EventWriter<MoveRequestEvent>
) {
    let mut translation = Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };

    for (controller, mut transform) in &mut param_set.p0().iter_mut() {
        let velocity = controller.max_speed * controller.intent;
        transform.translation += velocity.extend(0.0) * time.delta_seconds();
        translation = velocity.extend(0.0) * time.delta_seconds();
    }

    // No need to ping server and update camera if no change
    if !(translation.x == 0. && translation.y == 0.) {   
        for mut camera in &mut param_set.p1().iter_mut() {
            camera.translation += translation;
            
            // send movement to server
            move_request_event_writer.send(MoveRequestEvent(translation.x, translation.y));
        }
    }
}
