use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use tauri::{AppHandle, Emitter};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn handle_connection(mut stream: std::net::TcpStream, app: AppHandle) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    let mut temp_buf = [0; 4096];
    
    // Read headers first
    let mut header_end = None;
    while header_end.is_none() {
        let bytes_read = stream.read(&mut temp_buf)?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp_buf[..bytes_read]);
        
        // Find \r\n\r\n in buffer
        if let Some(pos) = find_subsequence(&buffer, b"\r\n\r\n") {
            header_end = Some(pos);
            break;
        }
    }
    
    let header_pos = match header_end {
        Some(pos) => pos,
        None => return Ok(()),
    };
    
    let header_str = String::from_utf8_lossy(&buffer[..header_pos]);
    
    // Handle CORS OPTIONS preflight request
    if header_str.starts_with("OPTIONS") {
        let response = "HTTP/1.1 204 No Content\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Access-Control-Allow-Methods: POST, OPTIONS, GET\r\n\
                        Access-Control-Allow-Headers: Content-Type, Authorization, X-Requested-With\r\n\
                        Access-Control-Max-Age: 86400\r\n\
                        Connection: close\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }
    
    if !header_str.starts_with("POST") {
        let response = "HTTP/1.1 405 Method Not Allowed\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Connection: close\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }
    
    // Find Content-Length
    let mut content_length = 0;
    for line in header_str.lines() {
        if line.to_lowercase().starts_with("content-length:") {
            if let Some(val_str) = line.split(':').nth(1) {
                if let Ok(len) = val_str.trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }
    }
    
    // Read the remaining body until we have content_length bytes
    let body_start = header_pos + 4;
    while buffer.len() < body_start + content_length {
        let bytes_read = stream.read(&mut temp_buf)?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp_buf[..bytes_read]);
    }
    
    // Extract JSON payload
    if body_start + content_length <= buffer.len() {
        let body_bytes = &buffer[body_start..body_start + content_length];
        let body_str = String::from_utf8_lossy(body_bytes);
        
        // ponytail: Emit events without intermediate abstractions
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
            let _ = app.emit("new-sscm-data", json);
        }
    }
    
    let response = "HTTP/1.1 200 OK\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Content-Type: application/json\r\n\
                    Connection: close\r\n\r\n\
                    {\"status\":\"success\"}";
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn start_http_listener(app: AppHandle) {
    thread::spawn(move || {
        // Bind listener. If port is in use, try logging or ignore
        if let Ok(listener) = TcpListener::bind("127.0.0.1:14120") {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let app_clone = app.clone();
                    thread::spawn(move || {
                        let _ = handle_connection(stream, app_clone);
                    });
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            start_http_listener(app_handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

