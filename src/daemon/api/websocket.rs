use opm::agent::registry::AgentRegistry;
use opm::agent::types::{AgentInfo, AgentStatus, ConnectionType};
use rocket::tokio;
use rocket::tokio::net::{TcpListener, TcpStream};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentMessage {
    /// Agent registration message
    Register {
        id: String,
        name: String,
        hostname: Option<String>,
        api_endpoint: Option<String>,
    },
    /// Heartbeat/ping message
    Heartbeat {
        id: String,
    },
    /// Response message
    Response {
        success: bool,
        message: String,
    },
    /// Ping message from server to agent
    Ping,
    /// Pong response from agent
    Pong,
}

/// Start the WebSocket server for agent connections
pub async fn start_websocket_server(
    address: String,
    registry: Arc<AgentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&address).await?;
    log::info!("[WebSocket] Server listening on {}", address);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                log::info!("[WebSocket] New connection from {}", addr);
                let registry = Arc::clone(&registry);
                
                tokio::spawn(async move {
                    if let Err(e) = handle_agent_connection(stream, registry).await {
                        log::error!("[WebSocket] Connection error from {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                log::error!("[WebSocket] Failed to accept connection: {}", e);
            }
        }
    }
}

/// Handle a single agent WebSocket connection
async fn handle_agent_connection(
    stream: TcpStream,
    registry: Arc<AgentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let mut agent_id: Option<String> = None;

    // Handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<AgentMessage>(&text) {
                    Ok(agent_msg) => {
                        match agent_msg {
                            AgentMessage::Register { id, name, hostname, api_endpoint } => {
                                log::info!("[WebSocket] Agent registration: {} ({})", name, id);
                                
                                let agent_info = AgentInfo {
                                    id: id.clone(),
                                    name: name.clone(),
                                    hostname,
                                    status: AgentStatus::Online,
                                    connection_type: ConnectionType::In,
                                    last_seen: std::time::SystemTime::now(),
                                    connected_at: std::time::SystemTime::now(),
                                    api_endpoint,
                                };
                                
                                registry.register(agent_info);
                                agent_id = Some(id);
                                
                                // Send success response
                                let response = AgentMessage::Response {
                                    success: true,
                                    message: "Agent registered successfully".to_string(),
                                };
                                
                                if let Ok(response_json) = serde_json::to_string(&response) {
                                    let _ = ws_sender.send(Message::Text(response_json)).await;
                                }
                            }
                            AgentMessage::Heartbeat { id } => {
                                log::debug!("[WebSocket] Heartbeat from agent {}", id);
                                
                                if registry.update_heartbeat(&id) {
                                    // Send pong response
                                    let response = AgentMessage::Response {
                                        success: true,
                                        message: "Heartbeat received".to_string(),
                                    };
                                    
                                    if let Ok(response_json) = serde_json::to_string(&response) {
                                        let _ = ws_sender.send(Message::Text(response_json)).await;
                                    }
                                } else {
                                    // Agent not found in registry
                                    let response = AgentMessage::Response {
                                        success: false,
                                        message: "Agent not found".to_string(),
                                    };
                                    
                                    if let Ok(response_json) = serde_json::to_string(&response) {
                                        let _ = ws_sender.send(Message::Text(response_json)).await;
                                    }
                                    
                                    // Close connection
                                    break;
                                }
                            }
                            AgentMessage::Pong => {
                                log::debug!("[WebSocket] Pong received from agent");
                                // Update last_seen time
                                if let Some(ref id) = agent_id {
                                    registry.update_heartbeat(id);
                                }
                            }
                            _ => {
                                log::warn!("[WebSocket] Unexpected message type");
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[WebSocket] Failed to parse message: {}", e);
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                // Respond to ping with pong
                let _ = ws_sender.send(Message::Pong(data)).await;
            }
            Ok(Message::Pong(_)) => {
                // Update heartbeat on pong
                if let Some(ref id) = agent_id {
                    registry.update_heartbeat(id);
                }
            }
            Ok(Message::Close(_)) => {
                log::info!("[WebSocket] Agent disconnected");
                break;
            }
            Err(e) => {
                log::error!("[WebSocket] Error receiving message: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup: unregister agent on disconnect
    if let Some(id) = agent_id {
        log::info!("[WebSocket] Unregistering agent {}", id);
        registry.unregister(&id);
    }

    Ok(())
}
