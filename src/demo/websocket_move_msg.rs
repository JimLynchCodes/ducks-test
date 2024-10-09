use std::{io::ErrorKind, net::TcpStream};

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use serde::{Deserialize, Serialize};
use tungstenite::{connect, http::Response, stream::MaybeTlsStream, Message, WebSocket};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<MoveRequestEvent>();

    // app.add_systems(Update, recv_ws_msg); // System to handle WebSocket messages
    app.add_systems(Update, move_request_bevy_event_listener);
}

use thiserror::Error;

use super::websocket_connect::WebSocketClient;

#[derive(Event)]
pub struct MoveRequestEvent(pub f32, pub f32);

// Listens for bevy events for ws messages and fires them off to the server
fn move_request_bevy_event_listener(
    mut ev_join_request: EventReader<MoveRequestEvent>,
    mut entities_with_client: Query<(&mut WebSocketClient,)>,
) {
    for ev in ev_join_request.read() {
        println!("heard move request bevy event");
        for mut client in entities_with_client.iter_mut() {
            println!("sending move request ws msg");
            let message = build_move_request_msg(ev.0.clone(), ev.1.clone());

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

// fn recv_ws_msg(mut q: Query<(&mut WebSocketClient,)>, mut commands: Commands) {
//     for (mut client,) in q.iter_mut() {
//         match client.0 .0.read() {
//             Ok(m) => {
//                 info!("Received message ws move {m:?}");
//                 // send_info(q);
//             }
//             Err(tungstenite::Error::Io(e)) if e.kind() == ErrorKind::WouldBlock => { /* ignore */ }
//             Err(e) => warn!("error receiving: {e}"),
//         }
//     }
// }

#[derive(serde::Serialize)]
struct MoveRequestData {
    x_direction: f32,
    y_direction: f32,
}

#[derive(serde::Serialize)]
struct MoveRequest {
    action_type: String,
    data: MoveRequestData,
}

fn build_move_request_msg(x_direction: f32, y_direction: f32) -> String {
    let join_request_hardcoded = MoveRequest {
        action_type: "move".to_string(),
        data: MoveRequestData {
            x_direction,
            y_direction,
        },
    };

    serde_json::ser::to_string(&join_request_hardcoded).unwrap_or_else(|_op| {
        println!("Couldn't convert You Quacked struct to string");
        "".to_string()
    })
}
