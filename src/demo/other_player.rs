use std::{io::ErrorKind, net::TcpStream};

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use serde::{Deserialize, Serialize};
use tungstenite::{connect, http::Response, stream::MaybeTlsStream, Message, WebSocket};



pub(super) fn plugin(app: &mut App) {
  
}
