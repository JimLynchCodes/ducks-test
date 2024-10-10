use bevy::prelude::*;
use bevy::render::texture::{ImageLoaderSettings, ImageSampler};
use serde::Deserialize;
use virtual_joystick::{
    create_joystick, JoystickFloating, JoystickInvisible, NoAction, VirtualJoystickEvent,
    VirtualJoystickPlugin,
};

use crate::demo::other_player::{unpack_duck_color, NewJoinerData};
use crate::{
    asset_tracking::LoadResource,
    demo::{player_animation::PlayerAnimation, movement::MovementController},
    screens::Screen,
};

use super::websocket_connect::YouJoinedWsReceived;

#[derive(Resource)]
pub struct QuackAudio {
    pub sound_handle: Handle<AudioSource>, // Handle for the loaded sound
}

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();
    app.register_type::<Player>();

    app.add_plugins(VirtualJoystickPlugin::<String>::default());
    app.add_systems(Startup, create_joystick_scene);
    app.add_systems(Update, handle_joystick_or_keyboard_input);
    app.add_systems(Startup, quack_sound_setup);
    app.add_systems(Update, spacebar_quack_system);
    app.add_systems(Update, you_joined_ws_msg_handler);
}

fn quack_sound_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut audio: ResMut<Assets<AudioSource>>,
) {
    let sound_handle = asset_server.load("audio/sound_effects/duck-quack.ogg"); // Adjust the path as necessary

    // Store the handle in a resource
    commands.insert_resource(QuackAudio { sound_handle });
}

fn spacebar_quack_system(
    mut commands: Commands,
    audio: Res<QuackAudio>,
    keyboard_input: Res<ButtonInput<KeyCode>>, // Input resource for key events
    mut query: Query<&mut Transform>,
    audio_assets: Res<Assets<AudioSource>>, // Query to find entities to affect
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Check if space was just pressed
        println!("Space pressed 1! Playing sound.");

        if let Some(_) = audio_assets.get(&audio.sound_handle) {
            // Spawn an audio source to play the sound
            commands.spawn(AudioSourceBundle {
                source: audio.sound_handle.clone(), // Clone the handle to use it
                ..Default::default()                // Use default values for other fields
            });
            println!("Space pressed 2! Playing sound.");
        } else {
            println!("Audio not loaded yet.");
        }
        // commands.spawn(AudioSourceBundle {
        //     source: audio.sound_handle.clone(), // Clone the handle to use it
        //     ..Default::default()                // Use default values for other fields
        // });

        // for mut transform in query.iter_mut() {
        //     transform.translation.y += 50.0; // Move entities up by 50 units
        //     println!("Space pressed! Moved entity up.");
        // }
    }
}

// fn you_joined_ws_msg_handler(mut event_reader: EventReader<YouJoinedWsReceived>) {

// for e in event_reader.read() {

//     info!("In level.rs handling the You joined event {:?}!", e);
// SpawnPlayer { max_speed: 400.0 }.apply(world);

// spawn_player()

// let Vec2 { x, y } = j.axis();

// // Joystick overrides the arrow keys! (but don't override of not touching joystick)
// if *x != 0. || *y != 0. {
//     intent.x = *x;
//     intent.y = *y;
// }
//     }

// }

fn create_joystick_scene(mut cmd: Commands, asset_server: Res<AssetServer>) {
    create_joystick(
        &mut cmd,
        "UniqueJoystick".to_string(),
        asset_server.load("Knob.png"),
        asset_server.load("Outline.png"),
        None,
        None,
        None,
        Vec2::new(75., 75.),
        Vec2::new(150., 150.),
        Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            position_type: PositionType::Absolute,
            left: Val::Percent(0.),
            bottom: Val::Percent(0.),
            ..default()
        },
        (JoystickInvisible, JoystickFloating),
        NoAction,
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

// spawn player
pub fn you_joined_ws_msg_handler(
    mut event_reader: EventReader<YouJoinedWsReceived>,
    mut commands: Commands,
    player_assets_op: Option<Res<PlayerAssets>>,
    mut asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    // friendly_name: String,
    // color: Color,
    // max_speed: f32,
) {
    if let Some(player_assets) = player_assets_op {
        for e in event_reader.read() {
            let you_joined_response_data =
                serde_json::from_value(e.data.clone()).unwrap_or_else(|op| {
                    info!("Failed to parse incoming websocket message: {}", op);
                    NewJoinerData {
                        player_uuid: "error".to_string(),
                        player_friendly_name: "error".to_string(),
                        color: "error".to_string(),
                        x_position: 0.,
                        y_position: 0.,
                    }
                });

            info!("In player.rs handling the You joined event {:?}!", e);
            let layout =
                TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
            let texture_atlas_layout = texture_atlas_layouts.add(layout);
            let player_animation = PlayerAnimation::new();

            let blue = "blue".to_string();

            commands
                .spawn((
                    Name::new("foo".to_string()),
                    Player,
                    SpriteBundle {
                        texture: player_assets.ducky.clone(),
                        transform: Transform {
                            scale: Vec2::splat(4.0).extend(2.0),
                            translation: Vec3::new(
                                you_joined_response_data.x_position,
                                you_joined_response_data.y_position,
                                10.0,
                            ),
                            ..Default::default()
                        },
                        sprite: Sprite {
                            color: unpack_duck_color(you_joined_response_data.color),
                            ..Default::default()
                        },
                        // Transform::from_scale(Vec2::splat(4.0).extend(2.0)),
                        ..Default::default()
                    },
                    TextureAtlas {
                        // color: match you_joined_response_data.color {
                        //     "blue" => Color::rgba(0.5, 0.5, 1.0),
                        //     _ => Color::rgba(0, 0, 0, 1)
                        // },
                        layout: texture_atlas_layout.clone(),
                        index: player_animation.get_atlas_index(),
                    },
                    MovementController {
                        max_speed: 500.,
                        ..default()
                    },
                    player_animation,
                    StateScoped(Screen::Gameplay),
                ))
                .with_children(|parent| {
                    // Text that appears above the sprite
                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            you_joined_response_data.player_friendly_name, // The text to display
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"), // Load your font here
                                font_size: 25.0,
                                color: Color::WHITE,
                            },
                        ),
                        transform: Transform {
                            translation: Vec3::new(0.0, 17.0, 1.0), // Position the text above the sprite
                            scale: Vec3::splat(0.25),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });
        }
    }
}

fn handle_joystick_or_keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    mut joystick: EventReader<VirtualJoystickEvent<String>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    for j in joystick.read() {
        let Vec2 { x, y } = j.axis();

        // Joystick overrides the arrow keys! (but don't override of not touching joystick)
        if *x != 0. || *y != 0. {
            intent.x = *x;
            intent.y = *y;
        }
    }

    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAssets {
    #[dependency]
    pub ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl PlayerAssets {
    pub const PATH_DUCKY: &'static str = "images/ducky.png";
    pub const PATH_STEP_1: &'static str = "audio/sound_effects/step1.ogg";
    pub const PATH_STEP_2: &'static str = "audio/sound_effects/step2.ogg";
    pub const PATH_STEP_3: &'static str = "audio/sound_effects/step3.ogg";
    pub const PATH_STEP_4: &'static str = "audio/sound_effects/step4.ogg";
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                PlayerAssets::PATH_DUCKY,
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load(PlayerAssets::PATH_STEP_1),
                assets.load(PlayerAssets::PATH_STEP_2),
                assets.load(PlayerAssets::PATH_STEP_3),
                assets.load(PlayerAssets::PATH_STEP_4),
            ],
        }
    }
}

// //! Plugin handling the player character in particular.
// //! Note that this is separate from the `movement` module as that could be used
// //! for other characters as well.

// use bevy::{
//     ecs::{system::RunSystemOnce as _, world::Command},
//     prelude::*,
//     render::texture::{ImageLoaderSettings, ImageSampler},
// };
// use virtual_joystick::{
//     create_joystick, JoystickFloating, JoystickInvisible, NoAction, VirtualJoystickEvent,
//     VirtualJoystickPlugin,
// };

// use crate::{
//     asset_tracking::LoadResource,
//     demo::{
//         animation::PlayerAnimation,
//         movement::MovementController,
//     },
//     screens::Screen,
// };

// pub(super) fn plugin(app: &mut App) {
//     app.register_type::<Player>();
//     app.load_resource::<PlayerAssets>();

//     app.add_plugins(VirtualJoystickPlugin::<String>::default());
//     // Record directional input as movement controls.
//     app.add_systems(Startup, create_joystick_scene);

//     // Handles input from Keyboard AND Joystick as movement controls.
//     app.add_systems(Update, handle_joystick_or_keyboard_input);
// }

// fn create_joystick_scene(mut cmd: Commands, asset_server: Res<AssetServer>) {
//     // Spawn Virtual Joystick at horizontal center using helper function
//     create_joystick(
//         &mut cmd,
//         "UniqueJoystick".to_string(),
//         asset_server.load("Knob.png"),
//         asset_server.load("Outline.png"),
//         None,
//         None,
//         None,
//         Vec2::new(75., 75.),
//         Vec2::new(150., 150.),
//         Style {
//             width: Val::Percent(100.),
//             height: Val::Percent(100.),
//             position_type: PositionType::Absolute,
//             left: Val::Percent(0.),
//             bottom: Val::Percent(0.),
//             ..default()
//         },
//         (JoystickInvisible, JoystickFloating),
//         NoAction,
//     );
// }

// #[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
// #[reflect(Component)]
// pub struct Player;

// /// A command to spawn the player character.
// #[derive(Debug)]
// pub struct SpawnPlayer {
//     /// See [`MovementController::max_speed`].
//     pub max_speed: f32,
// }

// impl Command for SpawnPlayer {
//     fn apply(self, world: &mut World) {
//         world.run_system_once_with(self, spawn_player);
//     }
// }

// fn spawn_player(
//     In(config): In<SpawnPlayer>,
//     mut commands: Commands,
//     player_assets: Res<PlayerAssets>,
//     mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
// ) {
//     // A texture atlas is a way to split one image with a grid into multiple
//     // sprites. By attaching it to a [`SpriteBundle`] and providing an index, we
//     // can specify which section of the image we want to see. We will use this
//     // to animate our player character. You can learn more about texture atlases in
//     // this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
//     let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
//     let texture_atlas_layout = texture_atlas_layouts.add(layout);
//     let player_animation = PlayerAnimation::new();

//     commands.spawn((
//         Name::new("Player"),
//         Player,
//         SpriteBundle {
//             texture: player_assets.ducky.clone(),
//             transform: Transform {
//                 scale: Vec2::splat(4.0).extend(2.0),
//                 translation:  Vec3::new(0.0, 0.0, 10.0),
//                 ..Default::default()
//             },

//             // Transform::from_scale(Vec2::splat(4.0).extend(2.0)),

//             ..Default::default()
//         },
//         TextureAtlas {
//             layout: texture_atlas_layout.clone(),
//             index: player_animation.get_atlas_index(),
//         },
//         MovementController {
//             max_speed: config.max_speed,
//             ..default()
//         },
//         player_animation,
//         StateScoped(Screen::Gameplay),
//     ));
// }

// fn handle_joystick_or_keyboard_input(
//     input: Res<ButtonInput<KeyCode>>,
//     mut joystick: EventReader<VirtualJoystickEvent<String>>,
//     mut controller_query: Query<&mut MovementController, With<Player>>,
// ) {
//     // Collect directional input.
//     let mut intent = Vec2::ZERO;
//     if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
//         intent.y += 1.0;
//     }
//     if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
//         intent.y -= 1.0;
//     }
//     if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
//         intent.x -= 1.0;
//     }
//     if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
//         intent.x += 1.0;
//     }

//     for j in joystick.read() {
//         let Vec2 { x, y } = j.axis();

//         // Joystick overrides the arrow keys! (but don't override of not touching joystick)
//         if *x != 0. || *y != 0. {
//             intent.x = *x;
//             intent.y = *y;
//         }
//     }

//     let intent = intent.normalize_or_zero();

//     // Apply movement intent to controllers.
//     for mut controller in &mut controller_query {
//         controller.intent = intent;
//     }
// }

// #[derive(Resource, Asset, Reflect, Clone)]
// pub struct PlayerAssets {
//     // This #[dependency] attribute marks the field as a dependency of the Asset.
//     // This means that it will not finish loading until the labeled asset is also loaded.
//     #[dependency]
//     pub ducky: Handle<Image>,
//     #[dependency]
//     pub steps: Vec<Handle<AudioSource>>,
// }

// impl PlayerAssets {
//     pub const PATH_DUCKY: &'static str = "images/ducky.png";
//     pub const PATH_STEP_1: &'static str = "audio/sound_effects/step1.ogg";
//     pub const PATH_STEP_2: &'static str = "audio/sound_effects/step2.ogg";
//     pub const PATH_STEP_3: &'static str = "audio/sound_effects/step3.ogg";
//     pub const PATH_STEP_4: &'static str = "audio/sound_effects/step4.ogg";
// }

// impl FromWorld for PlayerAssets {
//     fn from_world(world: &mut World) -> Self {
//         let assets = world.resource::<AssetServer>();
//         Self {
//             ducky: assets.load_with_settings(
//                 PlayerAssets::PATH_DUCKY,
//                 |settings: &mut ImageLoaderSettings| {
//                     // Use `nearest` image sampling to preserve the pixel art style.
//                     settings.sampler = ImageSampler::nearest();
//                 },
//             ),
//             steps: vec![
//                 assets.load(PlayerAssets::PATH_STEP_1),
//                 assets.load(PlayerAssets::PATH_STEP_2),
//                 assets.load(PlayerAssets::PATH_STEP_3),
//                 assets.load(PlayerAssets::PATH_STEP_4),
//             ],
//         }
//     }
// }
