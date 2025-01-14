extern crate lifx_core as lifx;

mod bulbs;

use lifx::{BuildOptions, Message, RawMessage};
use std::{net::UdpSocket, sync::Arc, time::Duration};
use tauri::{AppHandle, Emitter, Manager, State};

const LIFX_BROADCAST_PORT: u16 = 56700;
const BROADCAST_ADDR: &str = "255.255.255.255";

#[derive(Clone)]
struct AppState {
    app_handle: AppHandle,
    bulb_manager: Arc<bulbs::Manager>,
}

#[tauri::command]
async fn discover_lights(state: State<'_, AppState>) -> Result<(), String> {
    println!("Discovering Lights");
    let socket = UdpSocket::bind(("0.0.0.0", LIFX_BROADCAST_PORT))
        .map_err(|e| format!("Failed to bind socket: {}", e))
        .unwrap();
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

    let mut buf = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((_, src_addr)) => {
                println!("Device discovered: {}", src_addr);
                state
                    .app_handle
                    .emit("device_discovered", src_addr)
                    .unwrap();
            }
            Err(e) => {
                println!("Discovery error: {}", e);
                break;
            }
        };
    }

    Ok(())
}

#[tauri::command]
async fn lights_off(state: State<'_, AppState>) -> Result<(), String> {
    // Create a socket for sending messages
    let socket =
        UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Failed to bind socket: {}", e))?;

    // Get the list of bulbs
    let bulbs = state.bulb_manager.bulbs.lock().unwrap();

    for bulb in bulbs.values() {
        let options = BuildOptions {
            target: Some(bulb.target),
            res_required: true,
            source: 0x72757374,
            ..Default::default()
        };
        let message = RawMessage::build(
            &options,
            Message::LightSetPower {
                level: 0,
                duration: 0,
            },
        )
        .unwrap();
        let bytes = message.pack().unwrap();

        // connect to the bulb
        socket
            .connect(bulb.addr)
            .map_err(|e| format!("Failed to connect to {}: {}", bulb.addr, e))?;

        // Send the message to the bulb
        socket
            .send_to(&bytes, &bulb.addr)
            .map_err(|e| format!("Failed to send message to {}: {}", bulb.addr, e))?;
    }
    Ok(())
}

#[tauri::command]
async fn lights_on(state: State<'_, AppState>) -> Result<(), String> {
    // Create a socket for sending messages
    let socket =
        UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Failed to bind socket: {}", e))?;

    // Get the list of bulbs
    let bulbs = state.bulb_manager.bulbs.lock().unwrap();

    for bulb in bulbs.values() {
        let options = BuildOptions {
            target: Some(bulb.target),
            res_required: true,
            source: 0x72757374,
            ..Default::default()
        };
        let message = RawMessage::build(
            &options,
            Message::LightSetPower {
                level: 1,
                duration: 0,
            },
        )
        .unwrap();
        let bytes = message.pack().unwrap();

        // connect to the bulb
        socket
            .connect(bulb.addr)
            .map_err(|e| format!("Failed to connect to {}: {}", bulb.addr, e))?;

        // Send the message to the bulb
        socket
            .send_to(&bytes, &bulb.addr)
            .map_err(|e| format!("Failed to send message to {}: {}", bulb.addr, e))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Starting Lifx Lab");

    tauri::Builder::default()
        .setup(|app| {
            let app_state = AppState {
                app_handle: app.handle().clone(),
                bulb_manager: Arc::new(bulbs::Manager::new().unwrap()),
            };

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
