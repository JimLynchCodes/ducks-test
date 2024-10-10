//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;
use rand::prelude::*;
use std::time::Duration;

use crate::{
    audio::SoundEffect,
    demo::{movement::MovementController, player::PlayerAssets},
    AppSet,
};

use super::{other_player::OtherPlayerAssets, websocket_connect::OtherPlayerMovedWsReceived};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.register_type::<OtherPlayerAnimation>();
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSet::TickTimers),
            (
                // update_animation_movement,
                // update_animation_from_bevy_other_player_joined_event,
                update_animation_atlas,
                trigger_step_sound_effect,
            )
                .chain()
                .run_if(resource_exists::<OtherPlayerAssets>)
                .in_set(AppSet::Update),
        ),
    );
}

// Update the sprite direction and animation state (idling/walking).
// (handling in other_player.rs now)
//
// fn update_animation_movement(
//     mut event_reader: EventReader<OtherPlayerMovedWsReceived>,
//     mut player_query: Query<(&MovementController, &mut Sprite, &mut OtherPlayerAnimation)>,
// ) {

    
//     for (controller, mut sprite, mut animation) in &mut player_query {
//         let dx = controller.intent.x;
//         if dx != 0.0 {
//             sprite.flip_x = dx < 0.0;
//         }

//         let animation_state = if controller.intent == Vec2::ZERO {
//             OtherPlayerAnimationState::Idling
//         } else {
//             OtherPlayerAnimationState::Walking
//         };
//         animation.update_state(animation_state);
//     }
// }

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut OtherPlayerAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&OtherPlayerAnimation, &mut TextureAtlas)>) {
    for (animation, mut atlas) in &mut query {
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut commands: Commands,
    player_assets: Res<OtherPlayerAssets>,
    mut step_query: Query<&OtherPlayerAnimation>,
) {
    for animation in &mut step_query {
        if animation.state == OtherPlayerAnimationState::Walking
            && animation.changed()
            && (animation.frame == 2 || animation.frame == 5)
        {
            let rng = &mut rand::thread_rng();
            let random_step = player_assets.steps.choose(rng).unwrap();
            commands.spawn((
                AudioBundle {
                    source: random_step.clone(),
                    settings: PlaybackSettings::DESPAWN,
                },
                SoundEffect,
            ));
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OtherPlayerAnimation {
    timer: Timer,
    frame: usize,
    state: OtherPlayerAnimationState,
}

#[derive(Reflect, PartialEq)]
pub enum OtherPlayerAnimationState {
    Idling,
    Walking,
}

impl OtherPlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 2;
    /// The duration of each idle frame.
    const IDLE_INTERVAL: Duration = Duration::from_millis(500);
    /// The number of walking frames.
    const WALKING_FRAMES: usize = 6;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(50);

    fn idling() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: OtherPlayerAnimationState::Idling,
        }
    }

    fn walking() -> Self {
        Self {
            timer: Timer::new(Self::WALKING_INTERVAL, TimerMode::Repeating),
            frame: 0,
            state: OtherPlayerAnimationState::Walking,
        }
    }

    pub fn new() -> Self {
        Self::idling()
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
        self.frame = (self.frame + 1)
            % match self.state {
                OtherPlayerAnimationState::Idling => Self::IDLE_FRAMES,
                OtherPlayerAnimationState::Walking => Self::WALKING_FRAMES,
            };
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: OtherPlayerAnimationState) {
        if self.state != state {
            match state {
                OtherPlayerAnimationState::Idling => *self = Self::idling(),
                OtherPlayerAnimationState::Walking => *self = Self::walking(),
            }
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.finished()
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        match self.state {
            OtherPlayerAnimationState::Idling => self.frame,
            OtherPlayerAnimationState::Walking => 6 + self.frame,
        }
    }
}
