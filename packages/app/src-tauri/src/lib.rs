extern crate lifx_core as lifx;

use lifx::{BuildOptions, Message, RawMessage};
use serde::Serialize;
use std::{net::UdpSocket, time::Duration};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::bulbs;

const LIFX_PORT: u16 = 8089;
const LIFX_BROADCAST_PORT: u16 = 56700;
const BROADCAST_ADDR: &str = "255.255.255.255";

#[derive(Debug, Serialize, Clone)]
struct DiscoveredDevice {
    ip: String,
    target: u64,
    service_types: Vec<u8>,
    port: u16,
}

struct AppState {
    app_handle: AppHandle,
    socket: UdpSocket,
}

#[tauri::command]
async fn discover_lights(state: State<'_, AppState>) -> Result<(), String> {
    println!("discovering lights");
    let socket = &state.socket;

    //initialize socket properties
    socket
        .set_broadcast(true)
        .map_err(|e| format!("Failed to set broadcast: {}", e))?;
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

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

    // Receive packets
    let mut buf = [0u8; 1024];
    loop {
        match state.socket.recv_from(&mut buf) {
            Ok((_, src_addr)) => {
                println!("Device discovered: {}", src_addr);
                state
                    .app_handle
                    .emit("device_discovered", src_addr)
                    .unwrap();

                // if let Ok(raw_msg) = RawMessage::unpack(&buf[..size]) {
                //     if let Ok(Message::StateService(StateService { port, service })) =
                //         raw_msg.decode()
                //     {
                //         let device = DiscoveredDevice {
                //             ip: src_addr.ip().to_string(),
                //             target: raw_msg.frame_addr.target,
                //             service_types: vec![service],
                //             port,
                //         };
                //         println!("Device discovered: {device:?}");
                //         state.app_handle.emit("device_discovered", device).unwrap();
                //     }
                // }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout reached
                break;
            }
            Err(e) => {
                return Err(format!("Error receiving response: {}", e));
            }
        };
    }

    println!("Discovery complete");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Starting Lifx Lab");

    tauri::Builder::default()
        .setup(|app| {
            let app_state = AppState {
                app_handle: app.handle().clone(),
                socket: UdpSocket::bind(("0.0.0.0", LIFX_PORT))
                    .map_err(|e| format!("Failed to bind socket: {}", e))
                    .unwrap(),
            };

            app.manage(app_state);

            bulbs::Manager::new().unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![discover_lights])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
