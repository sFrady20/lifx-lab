extern crate lifx_core as lifx;

mod bulbs;

use lifx::{BuildOptions, Message, RawMessage, HSBK};
use std::{net::UdpSocket, sync::Mutex};
use tauri::{Manager, State};

struct AppState {
    bulb_manager: bulbs::Manager,
}

#[tauri::command]
async fn broadcast(
    state: State<'_, Mutex<AppState>>,
    message: lifx::Message,
) -> Result<(), String> {
    let state = state.lock().unwrap();

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
        let raw_message = RawMessage::build(&options, message.clone()).unwrap();
        let bytes = raw_message.pack().unwrap();

        // Connect to the bulb
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
async fn lights_off(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    broadcast(
        state,
        Message::LightSetPower {
            level: 0,
            duration: 1,
        },
    )
    .await
}

#[tauri::command]
async fn lights_on(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    broadcast(
        state,
        Message::LightSetPower {
            level: 1,
            duration: 0,
        },
    )
    .await
}

#[tauri::command]
async fn lights_set_color(
    state: State<'_, Mutex<AppState>>,
    h: u16,
    s: u16,
    b: u16,
) -> Result<(), String> {
    broadcast(
        state,
        Message::LightSetColor {
            reserved: 0,
            color: HSBK {
                hue: h,
                saturation: s,
                brightness: b,
                kelvin: 0,
            },
            duration: 0,
        },
    )
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Starting Lifx Lab");

    tauri::Builder::default()
        .setup(|app| {
            let app_state = AppState {
                bulb_manager: bulbs::Manager::new().unwrap(),
            };

            app.manage(Mutex::new(app_state));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            lights_on,
            lights_off,
            lights_set_color
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
