use std::{io::ErrorKind, net::TcpStream};

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use serde::{Deserialize, Serialize};
use tungstenite::{connect, http::Response, stream::MaybeTlsStream, Message, WebSocket};

#[derive(Serialize, Deserialize)]
struct TransformData {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

pub(super) fn plugin(app: &mut App) {
  
    app.add_event::<WebSocketConnectionEvents>();
    app.add_event::<JoinRequestEvent>();
    app.add_systems(Update, check_connection_input);
    app.add_systems(Update, setup_connection);
    app.add_systems(Update, handle_tasks);
    // app.add_systems(Update, send_info); // System to handle WebSocket messages
    app.add_systems(Update, recv_ws_msg); // System to handle WebSocket messages
    app.add_systems(Update, listen_for_bevy_events_for_ws_messages);
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

#[derive(Event)]
pub struct JoinRequestEvent(pub String);

// Listens for bevy events for ws messages and fires them off to the server
fn listen_for_bevy_events_for_ws_messages(
    mut ev_join_request: EventReader<JoinRequestEvent>,
    mut entities_with_client: Query<(&mut WebSocketClient,)>,
) {
    for ev in ev_join_request.read() {
        println!("heard join request bevy event");
        for mut client in entities_with_client.iter_mut() {
            println!("sending join request ws msg");
            let message = build_join_request_msg(ev.0.clone());

            match client.0 .0 .0.send(Message::text(message)) {
                Ok(_) => info!("Join request ws msg successfully sent to server!"),
                Err(tungstenite::Error::Io(e)) if e.kind() == ErrorKind::WouldBlock => { /* ignore */
                }
                Err(e) => {
                    warn!("Could not send the message: {e:?}");
                }
            }
        }
    }
}


fn recv_ws_msg(mut q: Query<(&mut WebSocketClient,)>, mut commands: Commands) {
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

#[derive(serde::Serialize)]
struct JoinRequestData {
    friendly_name: String,
}

#[derive(serde::Serialize)]
struct JoinRequest {
    action_type: String,
    data: JoinRequestData,
}

fn build_join_request_msg(
    friendly_name: String,
) -> String {
    let join_request_hardcoded = JoinRequest {
        action_type: "join".to_string(),
        data: JoinRequestData {
            friendly_name: friendly_name,
        },
    };

    let g = serde_json::ser::to_string(&join_request_hardcoded).unwrap_or_else(|_op| {
        println!("Couldn't convert You Quacked struct to string");
        "".to_string()
    });
    g
}
