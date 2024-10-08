use std::{io::ErrorKind, net::TcpStream};

use bevy::{
    audio::{AudioPlugin, SpatialScale},
    ecs::world::CommandQueue,
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use serde::{Deserialize, Serialize};
use tungstenite::{connect, http::Response, stream::MaybeTlsStream, Message, WebSocket};

use super::{
    other_player_animation::OtherPlayerAnimation,
    player::QuackAudio,
    websocket_connect::{
        OtherPlayerJoinedWsReceived, OtherPlayerMovedWsReceived, OtherPlayerQuackedWsReceived,
        S2CActionTypes,
    },
};

use crate::{
    asset_tracking::LoadResource,
    demo::{
        other_player_animation::{self, OtherPlayerAnimationState},
        player_animation::PlayerAnimation,
    },
    screens::Screen,
};

use bevy_kira_audio::*;
// OtherPlayerJoinedMsg

// OtherPlayerJoinedData

#[derive(Debug, Deserialize)]
pub struct NewJoinerData {
    pub player_uuid: String,
    pub player_friendly_name: String,
    pub color: String,
    pub x_position: f32,
    pub y_position: f32,
}

#[derive(Debug, Deserialize)]
pub struct MoveRequestData {
    pub x_direction: f32,
    pub y_direction: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MoveResponseData {
    pub player_uuid: String,
    pub player_friendly_name: String,
    pub color: String,
    pub old_x_position: f32,
    pub old_y_position: f32,
    pub new_x_position: f32,
    pub new_y_position: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuackResponseData {
    pub player_uuid: String,
    pub player_friendly_name: String,
    pub player_x_position: f32,
    pub player_y_position: f32,
    pub quack_pitch: f32,
}

// #[derive(Debug, Deserialize)]
// pub struct YouMovedReceivedWsMsg {
//     pub action_type: S2CActionTypes,
//     pub data: MoveResponseData,
// }

// Tag for the listener (e.g., player or camera)
#[derive(Component)]
struct Listener;

// Tag for sound emitters
#[derive(Component)]
struct SoundEmitter;

#[derive(Debug, Deserialize)]
pub struct OtherMovedReceivedWsMsg {
    pub action_type: S2CActionTypes,
    pub data: MoveResponseData,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct OtherPlayer;

#[derive(Resource, Asset, Reflect, Clone)]
pub struct OtherPlayerAssets {
    #[dependency]
    pub ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<bevy::audio::AudioSource>>,
}

impl OtherPlayerAssets {
    pub const PATH_DUCKY: &'static str = "images/ducky.png";
    pub const PATH_STEP_1: &'static str = "audio/sound_effects/step1.ogg";
    pub const PATH_STEP_2: &'static str = "audio/sound_effects/step2.ogg";
    pub const PATH_STEP_3: &'static str = "audio/sound_effects/step3.ogg";
    pub const PATH_STEP_4: &'static str = "audio/sound_effects/step4.ogg";
}

impl FromWorld for OtherPlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                OtherPlayerAssets::PATH_DUCKY,
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load(OtherPlayerAssets::PATH_STEP_1),
                assets.load(OtherPlayerAssets::PATH_STEP_2),
                assets.load(OtherPlayerAssets::PATH_STEP_3),
                assets.load(OtherPlayerAssets::PATH_STEP_4),
            ],
        }
    }
}

#[derive(Resource)]
pub struct SpatialAudio {
    /// The volume will change from `1` at distance `0` to `0` at distance `max_distance`
    pub max_distance: f32,
}


impl SpatialAudio {
    pub(crate) fn update(
        &self,
        receiver_transform: &GlobalTransform,
        emitters: &Query<(&GlobalTransform, &AudioEmitter)>,
        audio_instances: &mut Assets<AudioInstance>,
    ) {
        for (emitter_transform, emitter) in emitters {
            let sound_path = emitter_transform.translation() - receiver_transform.translation();
            let volume = (1. - sound_path.length() / self.max_distance)
                .clamp(0., 1.)
                .powi(2);

            let right_ear_angle = receiver_transform.right().angle_between(sound_path);
            let panning = (right_ear_angle.cos() + 1.) / 2.;

            for instance in emitter.instances.iter() {
                if let Some(instance) = audio_instances.get_mut(instance) {
                    instance.set_volume(volume as f64, AudioTween::default());
                    instance.set_panning(panning as f64, AudioTween::default());
                }
            }
        }
    }
}

// TODO - this not working?
pub(crate) fn run_spatial_audio(
    spatial_audio: Res<SpatialAudio>,
    receiver: Query<&GlobalTransform, With<AudioReceiver>>,
    emitters: Query<(&GlobalTransform, &AudioEmitter)>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Ok(receiver_transform) = receiver.get_single() {
        spatial_audio.update(receiver_transform, &emitters, &mut audio_instances);
    }
}

// pub(crate) fn cleanup_stopped_spatial_instances(
//     mut emitters: Query<&mut AudioEmitter>,
//     instances: ResMut<Assets<AudioInstance>>,
// ) {
//     for mut emitter in emitters.iter_mut() {
//         let handles = &mut emitter.instances;

//         handles.retain(|handle| {
//             if let Some(_instance) = instances.get(handle) {
//                 _instance.handle.state() != PlaybackState::Stopped
//             } else {
//                 true
//             }
//         });
//     }
// }

/// Component for audio emitters
///
/// Add [`Handle<AudioInstance>`]s to control their pan and volume based on emitter
/// and receiver positions.
#[derive(Component, Default)]
pub struct AudioEmitter {
    /// Audio instances that are played by this emitter
    ///
    /// The same instance should only be on one emitter.
    pub instances: Vec<Handle<AudioInstance>>,
}

/// Component for the audio receiver
///
/// Most likely you will want to add this component to your player or you camera.
/// The entity needs a [`Transform`] and [`GlobalTransform`]. The view direction of the [`GlobalTransform`]
/// will
#[derive(Component)]
pub struct AudioReceiver;

// ====

#[derive(Bundle)]
struct SpatialAudioBundle {
    audio_source: Handle<bevy_kira_audio::AudioSource>,
    transform: Transform,
    global_transform: GlobalTransform,
}

const AUDIO_SCALE: f32 = 1. / 100.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<OtherPlayer>();
    app.load_resource::<OtherPlayerAssets>();
    app.add_plugins(bevy_kira_audio::AudioPlugin);
    app.insert_resource(SpatialAudio { max_distance: 25. });
    // app.init_asset::<AudioSource>();

    // app.insert_resource(SpatialScale { scale: 1.0 }); // Scale of spatial audio
    app.add_systems(Update, other_player_joined_ws_msg_handler);
    app.add_systems(Update, other_player_moved_ws_msg_handler);
    app.add_systems(Update, other_player_quacked_handler);

    app.add_systems(Update, run_spatial_audio.run_if(resource_exists::<SpatialAudio>));
}

// app.add_plugins(DefaultPlugins.set(AudioPlugin {
//     default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
//     ..default()
// }));

// spawn player
pub fn other_player_joined_ws_msg_handler(
    mut event_reader: EventReader<OtherPlayerJoinedWsReceived>,
    mut commands: Commands,
    player_assets_op: Option<Res<OtherPlayerAssets>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    // friendly_name: String,
    // color: Color,
    // max_speed: f32,
) {
    if let Some(player_assets) = player_assets_op {
        for e in event_reader.read() {
            info!("other player joined!");

            #[derive(Debug, Deserialize)]
            pub struct NewJoinerData {
                pub player_uuid: String,
                pub player_friendly_name: String,
                pub color: String,
                pub x_position: f32,
                pub y_position: f32,
            }

            let other_player_joined_response_data = serde_json::from_value(e.data.clone())
                .unwrap_or_else(|op| {
                    info!("Failed to parse incoming websocket message: {}", op);
                    NewJoinerData {
                        player_uuid: "error".to_string(),
                        player_friendly_name: "error".to_string(),
                        color: "error".to_string(),
                        x_position: 0.,
                        y_position: 0.,
                    }
                });

            info!(
                "In other_player.rs handling the Other Player joined event {:?}!",
                e
            );
            let layout =
                TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
            let texture_atlas_layout = texture_atlas_layouts.add(layout);
            let player_animation = PlayerAnimation::new();

            let parent_entity = (
                Name::new(other_player_joined_response_data.player_uuid),
                OtherPlayer,
                SpriteBundle {
                    texture: player_assets.ducky.clone(),
                    transform: Transform {
                        scale: Vec2::splat(4.0).extend(2.0),
                        translation: Vec3::new(
                            other_player_joined_response_data.x_position,
                            other_player_joined_response_data.y_position,
                            10.0,
                        ),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: unpack_duck_color(other_player_joined_response_data.color),
                        ..Default::default()
                    },
                    // Transform::from_scale(Vec2::splat(4.0).extend(2.0)),
                    ..Default::default()
                },
                TextureAtlas {
                    // color: match other_player_joined_response_data.color {
                    //     "blue" => Color::rgba(0.5, 0.5, 1.0),
                    //     _ => Color::rgba(0, 0, 0, 1)
                    // },
                    layout: texture_atlas_layout.clone(),
                    index: player_animation.get_atlas_index(),
                },
                // MovementController {
                //     max_speed: 500.,
                //     ..default()
                // },
                player_animation,
                StateScoped(Screen::Gameplay),
            );

            commands.spawn(parent_entity).with_children(|parent| {
                // Get the parent's scale
                // let parent_scale_x = parent.entity().scale.x;

                // let parent_entity = parent.parent_entity();

                // let parent_transform = commands.get_or_spawn(parent_entity);

                // parent_transform.
                // Ok(transform) => {
                //     transform.clone();
                //     // transform
                //     return 0;
                // },
                // Err(_) => 0, // Handle the case where the parent doesn't exist
                // };

                // println!("Parent entity: {:?}", parent_entity);

                // if let Some(parent_transform) =
                //     commands.get_component::<Transform>(parent_entity)
                // {
                // Get the parent's scale
                // let parent_scale = parent_transform.scale;

                // Determine the scale for the text
                // let text_scale_x = if parent_transform.scale.x == -1.0 {
                //     -1.0
                // } else {
                //     1.0
                // };

                // Text that appears above the sprite
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        other_player_joined_response_data.player_friendly_name, // The text to display
                        TextStyle {
                            font: asset_server.load("FiraSans-Bold.ttf"), // Load your font here
                            font_size: 25.0,
                            color: Color::WHITE,
                        },
                    ),
                    transform: Transform {
                        translation: Vec3::new(0.0, 17.0, 1.0), // Position the text above the sprite
                        scale: Vec3::new(0.25, 0.25, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            });
        }
    }
}

// spawn player
pub fn other_player_moved_ws_msg_handler(
    mut event_reader: EventReader<OtherPlayerMovedWsReceived>,

    // mut other_players: Query<(&OtherPlayer, &mut Sprite, &Name, &mut Transform)>, // friendly_name: String,
    mut other_players: Query<(&OtherPlayer, &mut Sprite, &Name, &mut Transform)>, // friendly_name: String,
                                                                                  // color: Color,
                                                                                  // max_speed: f32,
) {
    for e in event_reader.read() {
        info!("Handling other player moved bevy event");

        let other_player_moved_response_data = serde_json::from_value(e.data.clone())
            .unwrap_or_else(|op| {
                info!("Failed to parse incoming websocket message: {}", op);
                MoveResponseData {
                    player_uuid: "error".to_string(),
                    player_friendly_name: "error".to_string(),
                    color: "error".to_string(),
                    old_x_position: 0.,
                    old_y_position: 0.,
                    new_x_position: 0.,
                    new_y_position: 0.,
                }
            });

        info!(
            "In other_player.rs handling the Other Player moved event {:?}!",
            e
        );

        for (other_player, mut sprite, name, mut transform) in other_players.iter_mut() {
            // for (other_player, mut sprite, name, mut transform) in other_players.iter_mut() {
            info!("checking vs id map: {}", name.to_string());
            if name.to_string() == other_player_moved_response_data.player_uuid {
                println!(
                    "Found entity with id: {}",
                    other_player_moved_response_data.player_uuid
                );
                // move that entity's position

                transform.translation.x = other_player_moved_response_data.new_x_position;
                transform.translation.y = other_player_moved_response_data.new_y_position;

                let dx = other_player_moved_response_data.new_x_position
                    - other_player_moved_response_data.old_x_position;

                sprite.flip_x = dx < 0.;

                // other_player_animation.sprite.update_state(OtherPlayerAnimationState::Walking);

                // sprite.get_field(name)

                // if dx < 0. {
                //     // sprite.flip_y =
                //     // transform.scale.x = -1. * transform.scale.x.abs();
                // } else {
                //     transform.scale.x = transform.scale.x.abs();
                // }
            }
        }
        // }

        // let layout =
        //     TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
        // let texture_atlas_layout = texture_atlas_layouts.add(layout);
        // let player_animation = PlayerAnimation::new();

        // let blue = "blue".to_string();

        // commands
        //     .spawn((
        //         Name::new(other_player_joined_response_data.player_uuid),
        //         OtherPlayer,
        //         SpriteBundle {
        //             texture: player_assets.ducky.clone(),
        //             transform: Transform {
        //                 scale: Vec2::splat(4.0).extend(2.0),
        //                 translation: Vec3::new(
        //                     other_player_joined_response_data.x_position,
        //                     other_player_joined_response_data.y_position,
        //                     10.0,
        //                 ),
        //                 ..Default::default()
        //             },
        //             sprite: Sprite {
        //                 color: match &(other_player_joined_response_data.color) {
        //                     blue => Color::srgba(0.5, 0.5, 1.0, 1.),
        //                     _ => Color::WHITE, // Default color
        //                 },
        //                 ..Default::default()
        //             },
        //             // Transform::from_scale(Vec2::splat(4.0).extend(2.0)),
        //             ..Default::default()
        //         },
        //         TextureAtlas {
        //             // color: match other_player_joined_response_data.color {
        //             //     "blue" => Color::rgba(0.5, 0.5, 1.0),
        //             //     _ => Color::rgba(0, 0, 0, 1)
        //             // },
        //             layout: texture_atlas_layout.clone(),
        //             index: player_animation.get_atlas_index(),
        //         },
        //         // MovementController {
        //         //     max_speed: 500.,
        //         //     ..default()
        //         // },
        //         player_animation,
        //         StateScoped(Screen::Gameplay),
        //     ))
        //     .with_children(|parent| {
        //         // Text that appears above the sprite
        //         parent.spawn(Text2dBundle {
        //             text: Text::from_section(
        //                 other_player_joined_response_data.player_friendly_name, // The text to display
        //                 TextStyle {
        //                     font: asset_server.load("FiraSans-Bold.ttf"), // Load your font here
        //                     font_size: 25.0,
        //                     color: Color::WHITE,
        //                 },
        //             ),
        //             transform: Transform {
        //                 translation: Vec3::new(0.0, 17.0, 1.0), // Position the text above the sprite
        //                 scale: Vec3::splat(0.25),
        //                 ..Default::default()
        //             },
        //             ..Default::default()
        //         });
        //     });
    }
}

pub fn unpack_duck_color(color: String) -> Color {
    match color.as_str() {
        "blue" => Color::srgba(0.5, 0.5, 1.0, 1.),
        "red" => Color::srgba(0.5, 0.1, 0., 1.),
        "green" => Color::srgba(0., 0.9, 0., 1.),
        "white" => Color::WHITE,
        _ => Color::WHITE, // Default color
    }
}

fn other_player_quacked_handler(
    mut commands: Commands,
    // quack_audio: Res<QuackAudio>,
    mut event_reader: EventReader<OtherPlayerQuackedWsReceived>,
    audio_assets: Res<Assets<bevy_kira_audio::AudioSource>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    // mut player_query: Query<(&MovementController, &mut Sprite, &mut OtherPlayerAnimation)>,
) {
    for e in event_reader.read() {
        let other_player_quacked_response_data = serde_json::from_value(e.data.clone())
            .unwrap_or_else(|op| {
                info!("Failed to parse incoming websocket message: {}", op);
                QuackResponseData {
                    player_uuid: "error".to_string(),
                    player_friendly_name: "error".to_string(),
                    player_x_position: 0.,
                    player_y_position: 0.,
                    quack_pitch: 0.,
                }
            });

        info!(
            "Got the quack info! {:?}",
            other_player_quacked_response_data
        );

        // Get the position from the event data
        let quack_position = Vec3::new(
            other_player_quacked_response_data.player_x_position,
            other_player_quacked_response_data.player_y_position,
            0.0, // Assuming 2D game, z-coordinate is 0
        );

        // let audio_handle = asset_server.load("audio/sound_effects/duck-quack.ogg");

        // if let Some(audio_handle) = audio_assets.get(&quack_audio.sound_handle) {
        // Spawn an audio source to play the sound
        // commands.spawn()

        // Nonspacial version
        // commands.spawn(AudioSourceBundle {
        //     source: quack_audio.sound_handle.clone(), // Clone the handle to use it

        //     ..Default::default()                // Use default values for other fields
        // });

        commands
        .spawn(TransformBundle::from_transform(Transform::from_xyz(other_player_quacked_response_data.player_x_position
            , other_player_quacked_response_data.player_y_position
            , 0.0))) // Position emitter to the right
        .insert(SoundEmitter);

        let audio_handle = asset_server.load("audio/sound_effects/duck-quack.ogg");
        audio.play(audio_handle);

    

        // commands.spawn(SpatialAudioBundle {
        //     audio_source: audio_handle.clone(),
        //     transform: Transform::from_translation(Vec3::new(
        //         other_player_quacked_response_data.player_x_position,
        //         other_player_quacked_response_data.player_y_position,
        //         0.0,
        //     )),
        //     global_transform: GlobalTransform::default(),
        // });
        // // Play the audio at the position defined above
        // audio.play(audio_handle.clone());

        
        //     audio_handle,
        //     Transform {
        //         translation: Vec3::new(other_player_quacked_response_data.player_x_position,
        //             other_player_quacked_response_data.player_y_position, 0.0), // Position the audio in 3D space
        //         ..Default::default()
        //     },
        //     GlobalTransform::default(), // Required for spatial positioning
        // ));
        

        // commands.spawn((
        //     SpatialAudio {
        //         emitter_position: quack_position, // Position of the quacking player
        //         listener_position: listener_transform.translation, // Player's current position (listener)
        //         ..Default::default() // Other default values, such as attenuation and falloff
        //     },
        //     audio.sound_handle.clone(), // Play the quack sound
        //     PlaybackSettings::ONCE.with_pitch(other_player_quacked_response_data.quack_pitch), // Adjust the pitch dynamically
        // ));

        //     println!("Received quack message! Playing sound.");
        // } else {
        //     println!("Audio not loaded yet.");
        // }
    }
}

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
