use std::{io::ErrorKind, net::TcpStream};

// use avian3d::prelude::*; // completely unnecessary but I like physics;
use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use serde::{Deserialize, Serialize};
// use iyes_perf_ui::{entries::PerfUiBundle, PerfUiPlugin};
use tungstenite::{connect, http::Response, stream::MaybeTlsStream, WebSocket};

// use std::sync::{Arc, Mutex};

// use bevy::{
//     app::{App, Startup, Update},
//     color::Color,
//     math::Vec2,
//     prelude::{Commands, Component, Query, Res, ResMut, Resource, Transform},
//     sprite::{Sprite, SpriteBundle},
//     tasks::{AsyncComputeTaskPool, IoTaskPool, Task},
// };
// use tokio_tungstenite::{connect_async, tungstenite::Message};
// use futures_util::{SinkExt, StreamExt};
// // use std::sync::mpsc::channel;
// use crossbeam_channel::{unbounded, Sender, Receiver};
// use tokio::runtime::Runtime;
// use tokio_tungstenite::{connect_async, tungstenite::Message};
// // use futures_util::stream::stream::StreamExt;
// use futures::{SinkExt, StreamExt};
// use std::thread;
// use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Serialize, Deserialize)]
struct TransformData {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<Player>();
    // app.load_resource::<PlayerAssets>();

    // app.add_plugins(VirtualJoystickPlugin::<String>::default());
    // Record directional input as movement controls.
    // app.add_systems(Startup, create_joystick_scene);

    // Handles input from Keyboard AND Joystick as movement controls.
    // app.add_systems(Update, handle_joystick_or_keyboard_input);

    // app.insert_resource(WebSocketResource::default());
    //     url: "ws://127.0.0.1:8000".to_string(), // Example WebSocket URL
    // }); // Insert WebSocket resource
    app.add_event::<WebSocketConnectionEvents>();
    app.add_systems(Update, check_connection_input);
    app.add_systems(Update, setup_connection);
    app.add_systems(Update, handle_tasks);
    app.add_systems(Update, send_info); // System to handle WebSocket messages
    app.add_systems(Update, recv_info); // System to handle WebSocket messages
                                        // app.add_startup_system(start_websocket_connection.system());
                                        // app.add_systems(Startup, setup_websocket); // System to handle WebSocket messages
                                        // app.add_systems(Update, handle_websocket_messages); // System to handle WebSocket messages
                                        // app.add_systems(Update, handle_received_messages); // System to handle WebSocket messages
}

#[derive(Component)]
struct WebSocketClient(
    (
        WebSocket<MaybeTlsStream<TcpStream>>,
        Response<Option<Vec<u8>>>,
    ),
);

#[derive(Event)]
enum WebSocketConnectionEvents {
    SetupConnection,
}

#[derive(Debug, serde::Serialize)]
struct _FooData {
    friendly_name: String,
}

#[derive(Debug, serde::Serialize)]
struct _Foo {
    action_type: String,
    data: _FooData,
}

fn check_connection_input(
    input: Res<ButtonInput<KeyCode>>,
    mut ev_connect: EventWriter<WebSocketConnectionEvents>,
) {
    if input.just_pressed(KeyCode::Space) {
        // set up connection
        ev_connect.send(WebSocketConnectionEvents::SetupConnection);
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
enum ConnectionSetupError {
    #[error("IO")]
    Io(#[from] std::io::Error),
    #[error("WebSocket")]
    WebSocket(#[from] tungstenite::Error),
}

#[derive(Component)]
struct WebSocketConnectionSetupTask(
    #[allow(unused)] Task<Result<CommandQueue, ConnectionSetupError>>,
);

fn setup_connection(
    mut ev_connect: EventReader<WebSocketConnectionEvents>,
    mut commands: Commands,
) {
    for ev in ev_connect.read() {
        match ev {
            WebSocketConnectionEvents::SetupConnection => {
                info!("Setting up connection!");
                let pool = AsyncComputeTaskPool::get();
                let entity = commands.spawn_empty().id();
                let task = pool.spawn(async move {
                    let mut client = connect("ws://127.0.0.1:8000/ws")?;
                    match client.0.get_mut() {
                        MaybeTlsStream::Plain(p) => p.set_nonblocking(true)?,
                        MaybeTlsStream::Rustls(stream_owned) => {
                            stream_owned.get_mut().set_nonblocking(true)?
                        }
                        _ => todo!(),
                    };
                    info!("Connected successfully!");
                    let mut command_queue = CommandQueue::default();

                    command_queue.push(move |world: &mut World| {
                        world
                            .entity_mut(entity)
                            .insert(WebSocketClient(client))
                            // Task is complete, so remove task component from entity
                            .remove::<WebSocketConnectionSetupTask>();
                    });

                    Ok(command_queue)
                });
                commands
                    .entity(entity)
                    .insert(WebSocketConnectionSetupTask(task));
            }
        }
    }
}

/// This system queries for entities that have our Task<Transform> component. It polls the
/// tasks to see if they're complete. If the task is complete it takes the result, adds a
/// new [`Mesh3d`] and [`MeshMaterial3d`] to the entity using the result from the task's work, and
/// removes the task component from the entity.
fn handle_tasks(
    mut commands: Commands,
    mut transform_tasks: Query<&mut WebSocketConnectionSetupTask>,
) {
    for mut task in &mut transform_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            match result {
                Ok(mut commands_queue) => {
                    commands.append(&mut commands_queue);
                }
                Err(e) => {
                    info!("Connection failed with: {e:?}");
                }
            }
        }
    }
}


fn send_info(
    // some_data: Query<(&Transform,)>,
    // mut entities_with_client: Query<(&mut WebSocketClient,)>,
) {
    // for (mut client,) in entities_with_client.iter_mut() {
        // let transforms = &some_data.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
        // info!("Sending data: {transforms:?}");
        // match client
        // .0
        //  .0
        // .send(Message::Binary(bincode::serialize(transforms).unwrap()))
        // {
        //     Ok(_) => info!("Data successfully sent!"),
        //     Err(tungstenite::Error::Io(e)) if e.kind() == ErrorKind::WouldBlock => { /* ignore */ }
        //     Err(e) => {
        //         warn!("Could not send the message: {e:?}");
        //     }
        // }
    // }
}

fn recv_info(mut q: Query<(&mut WebSocketClient,)>) {
    for (mut client,) in q.iter_mut() {
        match client.0 .0.read() {
            Ok(m) => {
                info!("Received message {m:?}");
                // send_info(q);
            }
            Err(tungstenite::Error::Io(e)) if e.kind() == ErrorKind::WouldBlock => { /* ignore */ }
            Err(e) => warn!("error receiving: {e}"),
        }
    }
}

async fn _build_msg(// quacker_client_id: &str,
    // quackerClient: &ClientConnection,
    // quackerClientsGameData: &ClientsGameData,
) -> String {
    // let default_game_data = ClientGameData {
    //     client_id: "error".to_string(),
    //     x_pos: 0,
    //     y_pos: 0,
    //     radius: 0,
    //     friendly_name: "error".to_string(),
    //     color: "error".to_string(),
    //     quack_pitch: 0.,
    //     cracker_count: 0,
    // };

    // let gaurd = quackerClientsGameData.lock().await;

    // let sender_game_data = gaurd.get(quacker_client_id).unwrap_or_else(|| {
    //     println!("Couldn't find client with id: {}", quacker_client_id);
    //     &default_game_data
    // });

    // let quack_message_struct = YouQuackedMsg {
    //     action_type: OutgoingGameActionType::OtherPlayerQuacked,
    //     data: QuackResponseData {
    //         player_uuid: sender_game_data.client_id.to_string(),
    //         player_friendly_name: sender_game_data.friendly_name.clone(),
    //         player_x_position: sender_game_data.x_pos,
    //         player_y_position: sender_game_data.y_pos,
    //         quack_pitch: sender_game_data.quack_pitch,
    //     },
    // };

    let foobar = _Foo {
        action_type: "something".to_string(),
        data: _FooData {
            friendly_name: "bob".to_string(),
        },
    };

    serde_json::ser::to_string(&foobar).unwrap_or_else(|_op| {
        println!("Couldn't convert You Quacked struct to string");
        "".to_string()
    })
}
