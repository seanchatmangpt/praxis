use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug, thiserror::Error)]
pub enum TransportRefusal {
    #[error("WebSocket disconnected by peer")]
    Disconnected,
    #[error("WebSocket error: {0}")]
    WsError(#[from] axum::Error),
    #[error("Backpressure refused: channel full")]
    Backpressure,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Message processing refused")]
    ProcessingRefused,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct WireMessage {
    pub id: String,
    pub payload: String,
}

/// A transport layer that bridges a WebSocket with bounded mpsc channels
/// to ensure backpressure and properly handle disconnects.
pub async fn bridge_transport(
    socket: WebSocket,
    incoming_tx: mpsc::Sender<WireMessage>,
    mut outgoing_rx: mpsc::Receiver<WireMessage>,
) -> Result<(), TransportRefusal> {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Spawn a task to handle outgoing messages (from application to WebSocket)
    let mut send_task: JoinHandle<Result<(), TransportRefusal>> = tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            let text = serde_json::to_string(&msg)?;
            if ws_sender.send(Message::Text(text)).await.is_err() {
                return Err(TransportRefusal::Disconnected);
            }
        }
        Ok(())
    });

    // Spawn a task to handle incoming messages (from WebSocket to application)
    let mut recv_task: JoinHandle<Result<(), TransportRefusal>> = tokio::spawn(async move {
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    let parsed: WireMessage = match serde_json::from_str(&text) {
                        Ok(p) => p,
                        Err(e) => return Err(TransportRefusal::Serialization(e)),
                    };
                    
                    // Backpressure check: use try_send or send
                    // If the channel is full, try_send returns an error, giving backpressure.
                    if incoming_tx.try_send(parsed).is_err() {
                        return Err(TransportRefusal::Backpressure);
                    }
                }
                Ok(Message::Close(_)) => return Err(TransportRefusal::Disconnected),
                Ok(_) => {
                    // Ignore other message types for this integration
                }
                Err(e) => return Err(TransportRefusal::WsError(e)),
            }
        }
        Err(TransportRefusal::Disconnected)
    });

    // Wait for the first task to complete or fail.
    // This ensures that if either side disconnects or fails, both are terminated.
    let result = tokio::select! {
        res = (&mut send_task) => {
            recv_task.abort();
            match res {
                Ok(inner) => inner,
                Err(_) => Err(TransportRefusal::ProcessingRefused), // Join error
            }
        },
        res = (&mut recv_task) => {
            send_task.abort();
            match res {
                Ok(inner) => inner,
                Err(_) => Err(TransportRefusal::ProcessingRefused), // Join error
            }
        }
    };

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_refusal_display() {
        let err = TransportRefusal::Disconnected;
        assert_eq!(err.to_string(), "WebSocket disconnected by peer");
    }
}
