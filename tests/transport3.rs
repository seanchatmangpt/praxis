use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as TungsteniteMessage};

// Simulates the Lean 4 extraction of thermodynamic data (from prop_thermo3.lean)
// that the broker receives and routes.
const LEAN_THERMO_PAYLOAD: &str = r#"{
    "source": "prop_thermo3.lean",
    "event": "Manufactures.step",
    "thermodynamic_state": {
        "S": "entropy_value",
        "G": "capability_gradient",
        "F_S_G": true
    }
}"#;

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // The broker sends the thermodynamic data to the client.
    if socket.send(Message::Text(LEAN_THERMO_PAYLOAD.to_string())).await.is_err() {
        return; // stream dead early
    }
    
    // Wait for acknowledgment from client (optional, but good for stream health check)
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if let Message::Text(text) = msg {
                assert_eq!(text, "ACK");
            }
        }
    }
}

#[tokio::test]
async fn test_lean4_thermodynamic_broker_transport() {
    // 1. Spin up the Axum server (Rust broker)
    let app = Router::new().route("/ws", get(ws_handler));
    
    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind");
    let addr = listener.local_addr().unwrap();
    
    // Run the server in a background task
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 2. Connect the WebSocket client
    let ws_url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect WebSocket client");

    // 3. Mechanically prove the data arrives at the WebSocket client
    let msg = ws_stream.next().await
        .expect("Stream is dead: No message received")
        .expect("WebSocket error");

    match msg {
        TungsteniteMessage::Text(text) => {
            // Verify it's the expected Lean 4 thermodynamic data
            assert_eq!(text, LEAN_THERMO_PAYLOAD, "Data mismatch in broker transport");
        }
        _ => panic!("Expected text message"),
    }
    
    // Send ACK to ensure stream is still alive and bidirectionally healthy
    ws_stream.send(TungsteniteMessage::Text("ACK".to_string())).await.expect("Failed to send ACK");
}
