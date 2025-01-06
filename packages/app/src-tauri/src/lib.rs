extern crate lifx_core as lifx;
use lifx::{BuildOptions, Message, RawMessage};
use serde::Serialize;
use std::net::{SocketAddr, UdpSocket};
use std::option;
use std::time::Duration;

const LIFX_PORT: u16 = 8089;
const LIFX_BROADCAST_PORT: u16 = 56700;
const BROADCAST_ADDR: &str = "255.255.255.255";

struct AppState {
    devices: Vec<DiscoveredDevice>,
}

#[derive(Debug, Serialize)]
struct DiscoveredDevice {
    ip: String,
    target: u64,
    service_types: Vec<u8>,
    port: u16,
}

#[tauri::command]
async fn discover_lights() -> Result<Vec<DiscoveredDevice>, String> {
    let socket = UdpSocket::bind(("0.0.0.0", LIFX_PORT))
        .map_err(|e| format!("Failed to bind socket: {}", e))?;

    socket
        .set_broadcast(true)
        .map_err(|e| format!("Failed to set broadcast: {}", e))?;

    let options = lifx::BuildOptions {
        ..Default::default()
    };
    let message = RawMessage::build(&options, Message::GetService).unwrap();
    let bytes = message.pack().unwrap();

    // Send broadcast message
    let broadcast_addr = format!("{}:{}", BROADCAST_ADDR, LIFX_BROADCAST_PORT);
    socket
        .send_to(&bytes, &broadcast_addr)
        .map_err(|e| format!("Failed to send broadcast: {}", e))?;

    // Set read timeout
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    let mut devices = Vec::new();
    let mut buf = [0u8; 1024];

    // Listen for responses
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, src_addr)) => {
                println!("Received packet from {}", src_addr);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout reached
                break;
            }
            Err(e) => {
                return Err(format!("Error receiving response: {}", e));
            }
        }
    }

    Ok(devices)
}

#[tauri::command]
fn lights_off() {
    println!("Lights on");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![lights_off, discover_lights])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
