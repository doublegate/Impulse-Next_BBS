//! WebSocket connection implementation

use crate::connection::{Connection, ConnectionError, ConnectionType};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};
use tracing::{debug, warn};

/// WebSocket connection wrapper
pub struct WebSocketConnection {
    /// The underlying WebSocket stream
    stream: Arc<Mutex<WebSocketStream<TcpStream>>>,
    /// Remote address
    remote_addr: String,
    /// Whether the connection is still active
    connected: Arc<Mutex<bool>>,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection from a TCP stream
    pub async fn new(stream: TcpStream, remote_addr: String) -> Result<Self, ConnectionError> {
        let ws_stream = accept_async(stream).await.map_err(|e| {
            ConnectionError::WebSocket(format!("Failed to accept WebSocket: {}", e))
        })?;

        Ok(Self {
            stream: Arc::new(Mutex::new(ws_stream)),
            remote_addr,
            connected: Arc::new(Mutex::new(true)),
        })
    }

    /// Send a JSON message
    pub async fn send_json<T: Serialize>(&mut self, message: &T) -> Result<(), ConnectionError> {
        let json = serde_json::to_string(message)
            .map_err(|e| ConnectionError::Encoding(format!("Failed to serialize JSON: {}", e)))?;

        self.send_text(&json).await
    }

    /// Receive and parse a JSON message
    pub async fn recv_json<T: for<'de> Deserialize<'de>>(
        &mut self,
    ) -> Result<Option<T>, ConnectionError> {
        match self.recv().await? {
            Some(data) => {
                let text = String::from_utf8(data)
                    .map_err(|e| ConnectionError::Encoding(format!("Invalid UTF-8: {}", e)))?;

                let message = serde_json::from_str(&text).map_err(|e| {
                    ConnectionError::Encoding(format!("Failed to parse JSON: {}", e))
                })?;

                Ok(Some(message))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl Connection for WebSocketConnection {
    fn connection_type(&self) -> ConnectionType {
        ConnectionType::WebSocket
    }

    fn remote_addr(&self) -> String {
        self.remote_addr.clone()
    }

    async fn send_text(&mut self, data: &str) -> Result<(), ConnectionError> {
        let mut stream = self.stream.lock().await;
        stream.send(Message::Text(data.into())).await.map_err(|e| {
            *self.connected.blocking_lock() = false;
            ConnectionError::WebSocket(format!("Failed to send text: {}", e))
        })?;

        debug!(remote = %self.remote_addr, "Sent text message");
        Ok(())
    }

    async fn send_bytes(&mut self, data: &[u8]) -> Result<(), ConnectionError> {
        let mut stream = self.stream.lock().await;
        stream
            .send(Message::Binary(data.to_vec().into()))
            .await
            .map_err(|e| {
                *self.connected.blocking_lock() = false;
                ConnectionError::WebSocket(format!("Failed to send bytes: {}", e))
            })?;

        debug!(remote = %self.remote_addr, bytes = data.len(), "Sent binary message");
        Ok(())
    }

    async fn recv(&mut self) -> Result<Option<Vec<u8>>, ConnectionError> {
        let mut stream = self.stream.lock().await;

        match stream.next().await {
            Some(Ok(msg)) => match msg {
                Message::Text(text) => {
                    debug!(remote = %self.remote_addr, "Received text message");
                    Ok(Some(text.as_bytes().to_vec()))
                }
                Message::Binary(data) => {
                    debug!(remote = %self.remote_addr, bytes = data.len(), "Received binary message");
                    Ok(Some(data.to_vec()))
                }
                Message::Ping(data) => {
                    // Respond to ping with pong
                    stream.send(Message::Pong(data)).await.ok();
                    debug!(remote = %self.remote_addr, "Received ping, sent pong");
                    // Continue receiving
                    drop(stream);
                    self.recv().await
                }
                Message::Pong(_) => {
                    debug!(remote = %self.remote_addr, "Received pong");
                    // Continue receiving
                    drop(stream);
                    self.recv().await
                }
                Message::Close(_) => {
                    debug!(remote = %self.remote_addr, "Received close frame");
                    *self.connected.lock().await = false;
                    Ok(None)
                }
                Message::Frame(_) => {
                    warn!(remote = %self.remote_addr, "Received unexpected frame");
                    drop(stream);
                    self.recv().await
                }
            },
            Some(Err(e)) => {
                warn!(remote = %self.remote_addr, error = %e, "WebSocket error");
                *self.connected.lock().await = false;
                Err(ConnectionError::WebSocket(format!("Receive error: {}", e)))
            }
            None => {
                debug!(remote = %self.remote_addr, "WebSocket stream ended");
                *self.connected.lock().await = false;
                Ok(None)
            }
        }
    }

    async fn close(&mut self) -> Result<(), ConnectionError> {
        let mut stream = self.stream.lock().await;
        stream
            .close(None)
            .await
            .map_err(|e| ConnectionError::WebSocket(format!("Failed to close: {}", e)))?;

        *self.connected.lock().await = false;
        debug!(remote = %self.remote_addr, "WebSocket connection closed");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        *self.connected.blocking_lock()
    }
}

/// WebSocket message protocol for BBS communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BbsMessage {
    /// Text output to display
    Output { text: String },

    /// Input from user
    Input { text: String },

    /// Command from user
    Command { command: String, args: Vec<String> },

    /// System notification
    Notification {
        message: String,
        level: NotificationLevel,
    },

    /// Session event
    Event { event: SessionEvent },

    /// Keep-alive ping
    Ping,

    /// Keep-alive pong
    Pong,
}

/// Notification level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

/// Session events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEvent {
    /// New mail notification
    NewMail { count: usize },

    /// Chat request
    ChatRequest { from_user: String },

    /// Session timeout warning
    TimeoutWarning { seconds_remaining: u64 },

    /// Session terminated
    Terminated { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbs_message_serialization() {
        let msg = BbsMessage::Output {
            text: "Hello, World!".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("output"));
        assert!(json.contains("Hello, World!"));

        let parsed: BbsMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            BbsMessage::Output { text } => assert_eq!(text, "Hello, World!"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_notification_levels() {
        let levels = vec![
            NotificationLevel::Info,
            NotificationLevel::Warning,
            NotificationLevel::Error,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let parsed: NotificationLevel = serde_json::from_str(&json).unwrap();
            // Just verify round-trip works
            let _ = parsed;
        }
    }

    #[test]
    fn test_session_event_serialization() {
        let event = SessionEvent::TimeoutWarning {
            seconds_remaining: 60,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("timeout_warning"));
        assert!(json.contains("60"));

        let parsed: SessionEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            SessionEvent::TimeoutWarning { seconds_remaining } => {
                assert_eq!(seconds_remaining, 60)
            }
            _ => panic!("Wrong event type"),
        }
    }
}
