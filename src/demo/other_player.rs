use std::{io::ErrorKind, net::TcpStream};

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use serde::{Deserialize, Serialize};
use tungstenite::{connect, http::Response, stream::MaybeTlsStream, Message, WebSocket};

use super::websocket_connect::{
    OtherPlayerJoinedWsReceived, OtherPlayerMovedWsReceived, S2CActionTypes,
};

use crate::{asset_tracking::LoadResource, demo::animation::PlayerAnimation, screens::Screen};

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

#[derive(Debug, Deserialize)]
pub struct YouMovedReceivedWsMsg {
    pub action_type: S2CActionTypes,
    pub data: MoveResponseData,
}

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
    pub steps: Vec<Handle<AudioSource>>,
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

pub(super) fn plugin(app: &mut App) {
    app.register_type::<OtherPlayer>();
    app.load_resource::<OtherPlayerAssets>();
    app.add_systems(Update, other_player_joined_ws_msg_handler);
    app.add_systems(Update, other_player_moved_ws_msg_handler);
}

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

    mut other_players: Query<(&OtherPlayer, &Name, &mut Transform)>, // friendly_name: String,
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

        for (other_player, name, mut transform) in other_players.iter_mut() {
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

                if dx < 0. {
                    transform.scale.x = -1. * transform.scale.x.abs();
                } else {
                    transform.scale.x = transform.scale.x.abs();
                }
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
