//! Spawn the main level.

use bevy::{ecs::world::Command, prelude::*, utils::info};

use super::websocket_connect::YouJoinedWsReceived;

pub(super) fn plugin(_app: &mut App) {
    // No setup required for this plugin.
    // It's still good to have a function here so that we can add some setup
    // later if needed.

    // _app.add_systems(Update, you_joined_ws_msg_handler);

    // TODO - listen for YouJoined ws message and THEN spawn the player...
    

    // TODO - listen for OtherPlayerJoined ws message and then spawn that player...

    
}

// fn you_joined_ws_msg_handler(mut event_reader: EventReader<YouJoinedWsReceived>) {
    
    
//     for e in event_reader.read() {
        
//         info!("In level.rs handling the You joined event {:?}!", e);
//         // SpawnPlayer { max_speed: 400.0 }.apply(world);

//         spawn_player()
        
//         // let Vec2 { x, y } = j.axis();

//         // // Joystick overrides the arrow keys! (but don't override of not touching joystick)
//         // if *x != 0. || *y != 0. {
//         //     intent.x = *x;
//         //     intent.y = *y;
//         // }
//     }

// }
// EventReader<WebSocketConnectionEvents>

/// A [`Command`] to spawn the level.
/// Functions that accept only `&mut World` as their parameter implement [`Command`].
/// We use this style when a command requires no configuration.
pub fn spawn_level(world: &mut World) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    
    // SpawnPlayer { max_speed: 400.0 }.apply(world);
}
