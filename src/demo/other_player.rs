use std::sync::{Arc, Mutex};

use bevy::{
    app::{App, Startup, Update},
    color::Color,
    math::Vec2,
    prelude::{Commands, Component, Query, Res, ResMut, Resource, Transform},
    sprite::{Sprite, SpriteBundle},
    tasks::{AsyncComputeTaskPool, IoTaskPool, Task},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
// use std::sync::mpsc::channel;
use crossbeam_channel::{unbounded, Sender, Receiver};
// use tokio::runtime::Runtime;
// use tokio_tungstenite::{connect_async, tungstenite::Message};
// // use futures_util::stream::stream::StreamExt;
// use futures::{SinkExt, StreamExt};
// use std::thread;
// use tokio::sync::mpsc::{self, Receiver, Sender};

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
    app.init_resource::<WebSocketState>();
                                                       // app.add_startup_system(start_websocket_connection.system());
    app.add_systems(Startup, setup_websocket); // System to handle WebSocket messages
    // app.add_systems(Update, handle_websocket_messages); // System to handle WebSocket messages
                                                        // app.add_systems(Update, handle_received_messages); // System to handle WebSocket messages
}

#[derive(Resource)]
struct WebSocketState {
    sender: Option<Sender<String>>,
    receiver: Option<Receiver<String>>,
}

// #[derive(Resource)]
struct WebSocketResource {
    tx: Sender<String>,
    rx: Receiver<String>,
}

impl Default for WebSocketState {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        WebSocketState { sender: Some(sender), receiver: Some(receiver) }
    }
}

fn setup_websocket(
    websocket_state: Res<WebSocketState>,
    // task_pools: Res<bevy::tasks::TaskPools>,
) {
    let sender = websocket_state.sender.clone();

    let task = async move {
        let url = "ws://localhost:8080";
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (mut write, mut read) = ws_stream.split();

        loop {
            tokio::select! {
                msg = read.next() => {
                    if let Some(Ok(msg)) = msg {
                        println!("Received: {}", msg);
                        // You might want to send this message back to the game logic
                        // sender.send(msg.to_string()).unwrap();
                    }
                }
                // You can add more async operations here if needed
            }
        }
    };

    // task_pool.spawn(task).detach();
}

// fn handle_websocket_messages(websocket_state: Res<WebSocketState>) {
//     if let Ok(message) = websocket_state.receiver.try_recv() {
//         println!("Sending message: {}", message);
//         // In a real scenario, you'd send this message over the WebSocket
//     }
// }

// fn handle_websocket_messages(websocket: Res<WebSocketResource>) {
//     if let Ok(message) = websocket.rx.try_recv() {
//         // Handle received message from the game logic
//         println!("Sending message: {}", message);
//         // In a real scenario, you'd send this message over the WebSocket
//     }
// }
// async fn websocket_task(url: String) {
//     let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
//     let (mut write, mut read) = ws_stream.split();

//     println!("hey:");

//     // Example: Sending a message to the WebSocket server
//     write.send(Message::Text("Hello WebSocket!".into())).await.expect("Failed to send message");

//     // Listening for messages from the WebSocket server
//     while let Some(message) = read.next().await {
//         match message {
//             Ok(msg) => {
//                 if let Message::Text(text) = msg {
//                     println!("Received: {}", text); // Print received message
//                 }
//             }
//             Err(e) => {
//                 eprintln!("Error: {}", e);
//             }
//         }
//     }
// }

// fn start_websocket_listener(
//     mut commands: Commands,
//     websocket_resource: Res<WebSocketResource>,
// ) {
//     // Spawn the WebSocket task
//     // tokio::spawn(websocket_task(websocket_resource.url.clone()));
//     let url = websocket_resource.url.clone();

//     let rt = Runtime::new().unwrap();
//     println!("starting");
    
//     let rt = Runtime::new().unwrap();
    
//     rt.spawn(async move {
//         // Connect to the WebSocket server
//         let url = "ws://127.0.0.1:8000"; // Replace with your WebSocket server
//         let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
//         println!("Connected to WebSocket!");

//         let (mut write, mut read) = ws_stream.split();

//         // Create a channel to send messages from Bevy systems
//         let (sender, mut receiver) = tokio::sync::mpsc::channel(100);

//         // websocket_resource.sender = Some(sender);

//         // Spawn a task to handle incoming WebSocket messages
//         tokio::spawn(async move {
//             while let Some(msg) = read.next().await {
//                 match msg {
//                     Ok(message) => {
//                         println!("Received: {:?}", message);
//                     }
//                     Err(e) => {
//                         eprintln!("Error reading from WebSocket: {}", e);
//                         break;
//                     }
//                 }
//             }
//         });

//         // Handle sending messages
//         tokio::spawn(async move {
//             while let Some(message) = receiver.recv().await {
//                 if write.send(message).await.is_err() {
//                     println!("Failed to send WebSocket message.");
//                     break;
//                 }
//             }
//         });
//     });
// }

fn handle_received_messages() {
    // This function could handle incoming messages,
    // such as updating Bevy entities or responding to user inputs.
}

// WebSocket resource to store the connection
// #[derive(Default, Resource)]
// struct WebSocketResource {
//     url: String,
//     // sender: Option<Sender<String>>, // Sender to send messages to the main thread
//     // receiver: Option<Arc<Mutex<Receiver<String>>>>,  // Receiver to receive messages

//                                     // sender: Option<Arc<Mutex<Sender<Message>>>>,
//                                     // received_messages: Vec<String>, // Store received messages
// }

// impl Default for WebSocketResource {
//     fn default() -> Self {
//         Self { receiver: None }
//     }
// }

// Component to store the player's name
#[derive(Component)]
struct OtherPlayerName(String);

// Component to link the text entity to the player
#[derive(Component)]
struct OtherPlayerNameText;

// Task to handle WebSocket connection asynchronously
// #[derive(Default)]
#[derive(Resource)]
struct WebSocketTask(Task<()>);

// Startup system to initiate WebSocket connection
// fn start_websocket_connection2(
//     mut websocket_resource: ResMut<WebSocketResource>,
//     // task_pool: Res<AsyncComputeTaskPool>,
//     mut commands: Commands,
// ) {
//     let (sender, receiver) = mpsc::channel(32);
//     websocket_resource.sender = Some(sender);
//     websocket_resource.receiver = Some(Arc::new(Mutex::new(receiver))); // Wrap receiver in Arc<Mutex>

//     // Clone the sender for the async task
//     let sender_clone = websocket_resource.sender.clone().unwrap();
//     let receiver_clone = Arc::clone(websocket_resource.receiver.as_ref().unwrap()); // Clone the Arc

//     tokio::spawn(async move {
//         // Replace with your WebSocket URL
//         let url = "ws://your_websocket_url"; 
//         let (ws_stream, _) = connect_async(url).await.unwrap();
//         let (mut ws_write, mut read) = ws_stream.split(); // Rename `write` to `ws_write`

//         // Simulating receiving messages from the WebSocket
//         while let Some(msg) = read.next().await {
//             match msg {
//                 Ok(message) => {
//                     if let Message::Text(text) = message {
//                         // Send the message to the main thread
//                         if let Err(_) = sender_clone.send(text).await {
//                             eprintln!("Sender has been dropped before sending.");
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("Error reading from WebSocket: {}", e);
//                     break;
//                 }
//             }
//         }
//     });

// }
// Startup system to initiate WebSocket connection
// fn start_websocket_connection0(
//     mut websocket: ResMut<WebSocketResource>,
//     // task_pool: Res<AsyncComputeTaskPool>,
//     mut commands: Commands,
// ) {
//     println!("Starting websocket connection!");
//     // Access Bevy's async compute task pool
//     // let task_pool = AsyncComputeTaskPool::get();

//     // let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel(32);
//     // websocket.sender = Some(tx);

//     // let sender = websocket.sender.take().unwrap(); // Take the sender out of the resource
//     // let (rx): Receiver<String> = websocket.receiver.take().unwrap(); // Take the receiver out of the resource

//     // Create a Tokio runtime for WebSocket tasks
//     let runtime = Runtime::new().expect("Failed to create Tokio runtime");

//     // let task = task_pool.spawn(async move {
//     //     runtime.block_on(async {
//     //         // Connect to the WebSocket server
//     //         let url = "ws://echo.websocket.org"; // Replace with your WebSocket server
//     //         let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
//     //         println!("Connected to WebSocket!");

//     //         let (mut write, mut read) = ws_stream.split();

//     //         // Create a channel to send messages from Bevy systems
//     //         let (sender, mut receiver) = tokio::sync::mpsc::channel(100);
//     //         let sender_arc = Arc::new(Mutex::new(sender));

//     //         // Spawn a task to handle incoming WebSocket messages
//     //         tokio::spawn(async move {
//     //             while let Some(msg) = read.next().await {
//     //                 match msg {
//     //                     Ok(message) => {
//     //                         if let Message::Text(text) = message {
//     //                             println!("Received: {}", text);
//     //                         }
//     //                     }
//     //                     Err(e) => {
//     //                         eprintln!("Error reading from WebSocket: {}", e);
//     //                         break;
//     //                     }
//     //                 }
//     //             }
//     //         });

//     //         // Handle sending messages from the receiver
//     //         while let Some(message) = receiver.recv().await {
//     //             if write.send(message).await.is_err() {
//     //                 println!("Failed to send WebSocket message.");
//     //                 break;
//     //             }
//     //         }
//     //     });
//     // });

//     // let task = task_pool.spawn(async move {
//     //     runtime.block_on(async {
//     // Connect to the WebSocket server
//     let url = "ws://127.0.0.1:8000/ws"; // Replace with your WebSocket server
//                                         // let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
//                                         // println!("Connected to WebSocket!");

//     // let (mut write, mut read) = ws_stream.split();  // Use split to separate sink and stream

//     // Check if sender and receiver are available
//     // let (sender, receiver) = match mpsc::channel(32) {
//     //     (s, r) => (Some(s), Some(r)),
//     // };

//     // let (sender, receiver) = mpsc::channel(32);

//     // websocket.sender = Some(sender);
//     // websocket.receiver = Some(receiver);

//     let sender = websocket.sender.take().unwrap();
//     let receiver = websocket.receiver.take().unwrap();

//     // Create a channel to send messages from Bevy systems
//     // let (sender, mut receiver) = tokio::sync::mpsc::channel(100);
//     // let sender_arc = Arc::new(Mutex::new(sender));

//     // Spawn a task to handle incoming WebSocket messages
//     // tokio::spawn(async move {
//     //     while let Some(msg) = read.next().await {
//     //         match msg {
//     //             Ok(message) => {
//     //                 if let Message::Text(text) = message {
//     //                     println!("Received: {}", text);

//     //                     if text == "new_joiner" {
//     //                         // spawn_duck(&commands);
//     //                         println!("new joiner!");

//     //                         // commands.spawn(bundle);
//     //                     }

//     //                 }
//     //             }
//     //             Err(e) => {
//     //                 eprintln!("Error reading from WebSocket: {}", e);
//     //                 break;
//     //             }
//     //         }
//     //     }
//     // });

//     // thread::spawn(move || {
//     //     let rt = Runtime::new().unwrap();
//     //     rt.block_on(async {
//     //         let (ws_stream, _) = connect_async(url).await.unwrap();
//     //         let (mut write, mut read) = ws_stream.split();

//     //         // Simulate receiving messages from the WebSocket
//     //         while let Some(msg) = read.next().await {
//     //             match msg {
//     //                 Ok(message) => {
//     //                     if let Message::Text(text) = message {
//     //                         // Send the message to the main thread
//     //                         if let Err(_) = sender.send(text).await {
//     //                             eprintln!("Receiver has been dropped");
//     //                         }
//     //                     }
//     //                 }
//     //                 Err(e) => {
//     //                     eprintln!("Error reading from WebSocket: {}", e);
//     //                     break;
//     //                 }
//     //             }
//     //         }
//     //     });
//     // });

//     thread::spawn(move || {
//         let rt = Runtime::new().unwrap();
//         rt.block_on(async {
//             // Replace with your WebSocket URL
//             // let url = "ws://your_websocket_url";
//             let (ws_stream, _) = connect_async(url).await.unwrap();
//             let (mut write, mut read) = ws_stream.split();

//             // Simulating receiving messages from the WebSocket
//             while let Some(msg) = read.next().await {
//                 match msg {
//                     Ok(message) => {
//                         if let Message::Text(text) = message {
//                             // Send the message to the main thread
//                             if let Err(_) = sender.send(text).await {
//                                 eprintln!("Receiver has been dropped");
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("Error reading from WebSocket: {}", e);
//                         break;
//                     }
//                 }
//             }
//         });
//     });

//     // Handle sending messages from the receiver
//     // while let Some(message) = receiver.recv().await {
//     //     if write.send(Message::Text(message)).await.is_err() {
//     //         println!("Failed to send WebSocket message.");
//     //         break;
//     //     }
//     // }
//     // });
//     // });

//     // // Store the task in the system
//     // commands.insert_resource(WebSocketTask(task));
// }

// // fn handle_received_messages(
// //     mut commands: Commands,
// //     mut websocket: Res<WebSocketResource>,
// // ) {
// //     // Ensure the receiver is available
// //     if let Some(receiver) = websocket.receiver.as_mut() {
// //         while let Ok(message) = receiver.try_recv() {
// //             if message == "new_joiner" {
// //                 // spawn_duck(commands);
// //                 println!("Spawning duck...")
// //             }
// //         }
// //     }
// // }

// fn handle_received_messages(
//     mut commands: Commands,
//     mut websocket: ResMut<WebSocketResource>, // Ensure we use ResMut
// ) {
//     // Check if the receiver is available and mutable
//     // if let Some(receiver) = websocket.receiver.as_mut() {
//     //     // Process incoming messages
//     //     while let Ok(message) = receiver.try_recv() {
//     //         if message == "new_joiner" {
//     //             // spawn_duck(&mut commands);

//     //             println!("spawning a duck?")
//     //         }
//     //     }
//     // }

//     // Access receiver as immutable; avoid moving it out of the resource
//     // if let Some(receiver) = &websocket.receiver {
//     //     let mut receiver = receiver.lock().unwrap(); // Lock the Mutex to access the receiver
//     //     while let Ok(message) = receiver.try_recv() {
//     //         if message == "new_joiner" {
//     //             // spawn_duck(commands);
//     //             println!("spawn duck???");
//     //         }
//     //     }
//     // } else {
//     //     eprintln!("Receiver is None");
//     // }
// }

// fn start_websocket_listener(mut websocket_resource: ResMut<WebSocketResource>) {
//     // Create a channel for sending messages from the WebSocket thread to the main thread
//     let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel(32); // Use Tokio's Sender and Receiver
//     websocket_resource.receiver = Some(rx); // Store the receiver in the resource

//     // Start the WebSocket listener in a new thread
//     thread::spawn(move || {
//         let rt = Runtime::new().unwrap();
//         rt.block_on(async {
//             // Replace this with your actual WebSocket URL
//             let (ws_stream, _) = connect_async("ws://your_websocket_url").await.unwrap();
//             let (mut write, mut read) = ws_stream.split();

//             while let Some(msg) = read.next().await {
//                 match msg {
//                     Ok(message) => {
//                         if let Message::Text(text) = message {
//                             // Send the message to the main thread
//                             if let Err(_) = tx.send(text).await {
//                                 // Handle the error if the receiver is dropped
//                                 eprintln!("Receiver has been dropped");
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("Error reading from WebSocket: {}", e);
//                         break;
//                     }
//                 }
//             }
//         });
//     });
// }



// System to handle sending WebSocket messages
// fn read_websocket_messages(keyboard_input: Res<Input<KeyCode>>, websocket: Res<WebSocketResource>) {
//     if let Some(sender) = &websocket.sender {
//         if keyboard_input.just_pressed(KeyCode::Space) {
//             let message = Message::Text("Hello WebSocket!".to_string());

//             // Clone sender to send the message asynchronously
//             let sender_clone = sender.clone();
//             tokio::spawn(async move {
//                 let mut sender_lock = sender_clone.lock().unwrap();
//                 sender_lock.send(message).await.unwrap();
//             });
//         }
//     }
// }

// fn handle_received_messages(
//     commands: &mut Commands,
//     websocket: Res<WebSocketResource>,
// ) {
//     // Ensure the receiver is available
//     if let Some(mut receiver) = &websocket..ref()receiver {
//         // Try to receive messages from the WebSocket
//         while let Ok(message) = receiver.try_recv() {
//             if message == "new_joiner" {
//                 spawn_duck(commands);
//             }
//         }
//     }
// }

// System to handle received WebSocket messages
// fn handle_received_messages(mut websocket: ResMut<WebSocketResource>) {
//     for message in &websocket.received_messages {
//         println!("Handling received message: {}", message);
//     }
//     websocket.received_messages.clear();
// }

// Startup system to initiate WebSocket connection
// fn start_websocket_connection(mut websocket: ResMut<WebSocketResource>) {
    // Create a new Tokio runtime
    // let rt = Runtime::new().unwrap();

    // rt.spawn(async move {
    //     // Connect to the WebSocket server
    //     let url = "ws://echo.websocket.org"; // Replace with your WebSocket server
    //     let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    //     println!("Connected to WebSocket!");

    //     let (mut write, mut read) = ws_stream.split();

    //     // Create a channel to send messages from Bevy systems
    //     let (sender, mut receiver) = tokio::sync::mpsc::channel(100);

    //     websocket.sender = Some(sender);

    //     // Spawn a task to handle incoming WebSocket messages
    //     tokio::spawn(async move {
    //         while let Some(msg) = read.next().await {
    //             match msg {
    //                 Ok(message) => {
    //                     println!("Received: {:?}", message);
    //                 }
    //                 Err(e) => {
    //                     eprintln!("Error reading from WebSocket: {}", e);
    //                     break;
    //                 }
    //             }
    //         }
    //     });

        // Handle sending messages
        // tokio::spawn(async move {
        //     while let Some(message) = receiver.recv().await {
        //         if write.send(tokio_tungstenite::tungstenite::Message::Text(message.clone())).await.is_err() {
        //             println!("Failed to send WebSocket message.");
        //             break;
        //         }
        //     }
        // });
    // });
// }

// // System to handle sending WebSocket messages
// fn read_websocket_messages(websocket: Res<WebSocketResource>) {
//     if let Some(sender) = &websocket.sender {
//         // if keyboard_input.just_pressed(KeyCode::Space) {
//         //     let message = Message::Text("Hello WebSocket!".to_string());

//         //     // Clone sender to send the message asynchronously
//         //     let sender_clone = sender.clone();
//         //     tokio::spawn(async move {
//         //         sender_clone.send(message).await.unwrap();
//         //     });
//         // }
//     }
// }
