extern crate lifx_core as lifx;

use lifx::{BuildOptions, Message, RawMessage};
use std::{net::UdpSocket, sync::Arc, time::Duration};
use tauri::{AppHandle, Emitter, Manager, State};

const LIFX_BROADCAST_PORT: u16 = 56700;
const BROADCAST_ADDR: &str = "255.255.255.255";

#[derive(Clone)]
struct AppState {
    app_handle: AppHandle,
    broadcast_socket: Arc<UdpSocket>,
}

#[tauri::command]
async fn discover_lights(state: State<'_, AppState>) -> Result<(), String> {
    println!("Discovering Lights");
    let socket = &state.broadcast_socket;

    // Set broadcast message
    let options = BuildOptions {
        ..Default::default()
    };
    let message = RawMessage::build(&options, Message::GetService).unwrap();
    let bytes = message.pack().unwrap();

    // Send broadcast message
    let broadcast_addr = format!("{}:{}", BROADCAST_ADDR, LIFX_BROADCAST_PORT);
    let _ = socket
        .send_to(&bytes, &broadcast_addr)
        .map_err(|e| format!("Failed to send broadcast: {}", e));

    println!("Discovery message sent");
    Ok(())
}

#[tauri::command]
async fn lights_off(_: State<'_, AppState>) -> Result<(), String> {
    println!("Turning lights off");
    Ok(())
}

#[tauri::command]
async fn lights_on(_: State<'_, AppState>) -> Result<(), String> {
    println!("Turning lights on");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Starting Lifx Lab");

    tauri::Builder::default()
        .setup(|app| {
            let broadcast_socket = UdpSocket::bind(("0.0.0.0", LIFX_BROADCAST_PORT))
                .map_err(|e| format!("Failed to bind socket: {}", e))
                .unwrap();
            broadcast_socket
                .set_broadcast(true)
                .map_err(|e| format!("Failed to set broadcast: {}", e))?;
            broadcast_socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .map_err(|e| format!("Failed to set timeout: {}", e))?;

            let app_state = AppState {
                app_handle: app.handle().clone(),
                broadcast_socket: Arc::new(broadcast_socket),
            };

            let app_state_for_listeners = app_state.clone();

            tauri::async_runtime::spawn(async move {
                let _ = listen_for_devices(app_state_for_listeners).await;
            });

            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            discover_lights,
            lights_on,
            lights_off
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn listen_for_devices(state: AppState) {
    let mut buf = [0u8; 1024];
    println!("Listening for devices");
    loop {
        println!("Waiting for response");
        match state.broadcast_socket.recv_from(&mut buf) {
            Ok((_, src_addr)) => {
                println!("Device discovered: {}", src_addr);
                state
                    .app_handle
                    .emit("device_discovered", src_addr)
                    .unwrap();
            }
            Err(e) => {
                println!("Discovery error: {}", e);
            }
        };
    }
}
