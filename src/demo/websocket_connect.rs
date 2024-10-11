use std::{io::ErrorKind, net::TcpStream};

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tungstenite::{connect, http::Response, stream::MaybeTlsStream, Message, WebSocket};

use strum_macros::EnumString;

// Client to Server types
#[derive(Debug, PartialEq, EnumString, Serialize)]
pub enum C2SActionTypes {
    #[strum(serialize = "join", serialize = "j")]
    Join,

    #[strum(serialize = "quack", serialize = "q")]
    Quack,

    #[strum(serialize = "move", serialize = "m")]
    Move,

    #[strum(serialize = "interact", serialize = "i")]
    Interact,

    #[strum(serialize = "empty", serialize = "e")]
    Empty, // used as a default in order to ignore invalid inputs without panicing
}

// Server to Client actions
#[derive(Debug, PartialEq, EnumString, Serialize, Clone, Deserialize)]
pub enum S2CActionTypes {
    #[strum(serialize = "you_joined", serialize = "yj")]
    YouJoined,
    #[strum(serialize = "other_player_joined", serialize = "opj")]
    OtherPlayerJoined,

    #[strum(serialize = "you_quacked", serialize = "yq")]
    YouQuacked,
    #[strum(serialize = "other_player_quacked", serialize = "opq")]
    OtherPlayerQuacked,

    #[strum(serialize = "you_moved", serialize = "ym")]
    YouMoved,
    #[strum(serialize = "other_player_moved", serialize = "opm")]
    OtherPlayerMoved,

    #[strum(serialize = "you_got_crackers", serialize = "ygc")]
    YouGotCrackers,
    #[strum(serialize = "other_player_got_crackers", serialize = "opgc")]
    OtherPlayerGotCrackers,

    #[strum(serialize = "you_died", serialize = "yd")]
    YouDied,
    #[strum(serialize = "other_player_died", serialize = "opd")]
    OtherPlayerGotDied,

    #[strum(serialize = "empty", serialize = "e")]
    Empty,
}

pub(super) fn plugin(app: &mut App) {
    app.add_event::<WebSocketConnectionEvents>();
    app.add_event::<YouJoinedWsReceived>();
    app.add_event::<OtherPlayerJoinedWsReceived>();
    app.add_event::<OtherPlayerMovedWsReceived>();
    app.add_event::<OtherPlayerQuackedWsReceived>();

    app.add_systems(Startup, actually_connect);
    app.add_systems(Update, setup_connection);
    app.add_systems(Update, handle_tasks);
    app.add_systems(Update, receive_ws_msg);

}

#[derive(Component)]
pub struct WebSocketClient(
    pub  (
        WebSocket<MaybeTlsStream<TcpStream>>,
        Response<Option<Vec<u8>>>,
    ),
);

#[derive(Event)]
enum WebSocketConnectionEvents {
    SetupConnection,
}

#[derive(Event, Debug, Clone)]
pub struct YouJoinedWsReceived {
    pub data: Value,
}

#[derive(Event, Debug, Clone)]
pub struct OtherPlayerJoinedWsReceived {
    pub data: Value,
}

#[derive(Event, Debug, Clone)]
pub struct OtherPlayerQuackedWsReceived {
    pub data: Value,
}

#[derive(Event, Debug, Clone)]
pub struct OtherPlayerMovedWsReceived {
    pub data: Value,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenericIncomingRequest {
    pub action_type: S2CActionTypes,
    pub data: Value,
}

fn actually_connect(
    _input: Res<ButtonInput<KeyCode>>,
    mut ev_connect: EventWriter<WebSocketConnectionEvents>,
) {
    ev_connect.send(WebSocketConnectionEvents::SetupConnection);
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

fn receive_ws_msg(
    mut q: Query<(&mut WebSocketClient,)>,
    mut bevy_event_writer_you_joined: EventWriter<YouJoinedWsReceived>,
    mut bevy_event_writer_other_player_joined: EventWriter<OtherPlayerJoinedWsReceived>,
    mut bevy_event_writer_other_player_quacked: EventWriter<OtherPlayerQuackedWsReceived>,
    mut bevy_event_writer_other_player_moved: EventWriter<OtherPlayerMovedWsReceived>,
) {
    for (mut client,) in q.iter_mut() {
        match client.0 .0.read() {
            Ok(m) => {
                info!("Received message ws connect {m:?}");

                let generic_msg =
                    serde_json::from_str(&m.to_text().unwrap()).unwrap_or_else(|op| {
                        info!("Failed to parse incoming websocket message: {}", op);
                        GenericIncomingRequest {
                            action_type: S2CActionTypes::Empty,
                            data: Value::Null,
                        }
                    });

                match generic_msg.action_type {
                    S2CActionTypes::YouJoined => {
                        info!("Received 'YouJoined' message from ws server!");
                        bevy_event_writer_you_joined.send(YouJoinedWsReceived{ data: generic_msg.data });
                    }
                    S2CActionTypes::OtherPlayerJoined => {
                        bevy_event_writer_other_player_joined.send(OtherPlayerJoinedWsReceived{ data: generic_msg.data });
                        info!("Received 'OtherPlayerJoined' message from ws server!");
                    }
                    S2CActionTypes::YouQuacked => {
                        // Basically ignored (bc quack sound already played before sending to server)
                        info!("Received 'YouQuacked' message from ws server!");
                    }
                    S2CActionTypes::OtherPlayerQuacked => {
                        bevy_event_writer_other_player_quacked.send(OtherPlayerQuackedWsReceived{ data: generic_msg.data });
                        info!("Received 'OtherPlayerQuacked' message from ws server!");
                    }
                    S2CActionTypes::YouMoved => {
                        // Basically ignored (bc you already moved before sending to server)
                        info!("Received 'YouMoved' message from ws server!");
                    }
                    S2CActionTypes::OtherPlayerMoved => {
                        bevy_event_writer_other_player_moved.send(OtherPlayerMovedWsReceived{ data: generic_msg.data });
                        info!("Received 'OtherPlayerMoved' message from ws server!");
                    }
                    S2CActionTypes::YouGotCrackers => {
                        info!("Received 'YouGotCrackers' message from ws server!");
                    }
                    S2CActionTypes::OtherPlayerGotCrackers => {
                        info!("Received 'OtherPlayerGotCrackers' message from ws server!");
                    }
                    S2CActionTypes::YouDied => {
                        info!("Received 'YouDied' message from ws server!");
                    }
                    S2CActionTypes::OtherPlayerGotDied => {
                        info!("Received 'OtherPlayerGotDied' message from ws server!");
                    }
                    S2CActionTypes::Empty => {
                        info!("Received 'Empty' message from ws server!");
                    }
                }
            }
            Err(tungstenite::Error::Io(e)) if e.kind() == ErrorKind::WouldBlock => { /* ignore */ }
            Err(e) => warn!("error receiving: {e}"),
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
