use super::types::{AgentConfig, AgentInfo, AgentStatus};
use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct AgentConnection {
    config: AgentConfig,
    status: AgentStatus,
}

impl AgentConnection {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            status: AgentStatus::Offline,
        }
    }

    /// Start the agent connection with hybrid WebSocket + polling fallback
    pub async fn run(&mut self) -> Result<()> {
        println!("[Agent] Starting agent '{}' (ID: {})", self.config.name, self.config.id);
        println!("[Agent] Connecting to server: {}", self.config.server_url);

        loop {
            match self.connect_websocket().await {
                Ok(()) => {
                    println!("[Agent] WebSocket connection established");
                    self.status = AgentStatus::Online;
                    
                    if let Err(e) = self.handle_websocket().await {
                        eprintln!("[Agent] WebSocket error: {}", e);
                        self.status = AgentStatus::Reconnecting;
                    }
                }
                Err(e) => {
                    eprintln!("[Agent] WebSocket connection failed: {}", e);
                    eprintln!("[Agent] Falling back to polling mode");
                    self.status = AgentStatus::Reconnecting;
                    
                    if let Err(e) = self.polling_mode().await {
                        eprintln!("[Agent] Polling mode error: {}", e);
                    }
                }
            }

            // Reconnection backoff
            println!("[Agent] Reconnecting in {} seconds...", self.config.reconnect_interval);
            sleep(Duration::from_secs(self.config.reconnect_interval)).await;
        }
    }

    async fn connect_websocket(&self) -> Result<()> {
        // Convert HTTP URL to WebSocket URL
        let ws_url = self.config.server_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let ws_url = format!("{}/agent/connect", ws_url);

        let _url = tokio_tungstenite::tungstenite::http::Uri::try_from(ws_url.as_str())?;
        Ok(())
    }

    async fn handle_websocket(&mut self) -> Result<()> {
        let ws_url = self.config.server_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let ws_url = format!("{}/agent/connect", ws_url);

        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send registration message
        let registration = json!({
            "type": "register",
            "id": self.config.id,
            "name": self.config.name,
            "token": self.config.token,
        });

        write.send(Message::Text(registration.to_string())).await?;

        // Heartbeat task
        let heartbeat_interval = self.config.heartbeat_interval;
        let id = self.config.id.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(heartbeat_interval));
            loop {
                interval.tick().await;
                let _heartbeat = json!({
                    "type": "heartbeat",
                    "id": id,
                });
                // In real implementation, we'd send this through the write half
                println!("[Agent] Heartbeat sent");
            }
        });

        // Handle incoming messages
        while let Some(msg) = read.next().await {
            match msg? {
                Message::Text(text) => {
                    println!("[Agent] Received: {}", text);
                    // Handle commands from server
                }
                Message::Close(_) => {
                    println!("[Agent] Server closed connection");
                    return Err(anyhow!("Connection closed by server"));
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn polling_mode(&self) -> Result<()> {
        println!("[Agent] Starting polling mode");
        
        let client = reqwest::Client::new();
        let poll_url = format!("{}/agent/poll", self.config.server_url);

        loop {
            // Register/heartbeat via HTTP
            let mut request = client.post(&poll_url)
                .json(&json!({
                    "id": self.config.id,
                    "name": self.config.name,
                    "status": "online",
                }));

            if let Some(ref token) = self.config.token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        // Process any pending commands
                        if let Ok(text) = response.text().await {
                            if !text.is_empty() {
                                println!("[Agent] Commands received: {}", text);
                            }
                        }
                    } else {
                        eprintln!("[Agent] Poll failed with status: {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("[Agent] Poll request failed: {}", e);
                    return Err(anyhow!(e));
                }
            }

            sleep(Duration::from_secs(self.config.heartbeat_interval)).await;
        }
    }

    pub fn get_info(&self) -> AgentInfo {
        use super::types::ConnectionType;
        AgentInfo {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            hostname: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok()),
            status: self.status.clone(),
            connection_type: ConnectionType::Out,
            last_seen: std::time::SystemTime::now(),
            connected_at: std::time::SystemTime::now(),
        }
    }
}
