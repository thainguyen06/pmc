use super::types::{AgentConfig, AgentInfo, AgentStatus};
use anyhow::{Result, anyhow};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

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

    /// Start the agent connection using HTTP polling
    pub async fn run(&mut self) -> Result<()> {
        println!("[Agent] Starting agent '{}' (ID: {})", self.config.name, self.config.id);
        println!("[Agent] Connecting to server: {}", self.config.server_url);

        loop {
            if let Err(e) = self.polling_mode().await {
                eprintln!("[Agent] Connection error: {}", e);
                self.status = AgentStatus::Reconnecting;
            }

            // Reconnection backoff
            println!("[Agent] Reconnecting in {} seconds...", self.config.reconnect_interval);
            sleep(Duration::from_secs(self.config.reconnect_interval)).await;
        }
    }

    async fn polling_mode(&mut self) -> Result<()> {
        println!("[Agent] Starting HTTP polling mode");
        
        let client = reqwest::Client::new();
        let register_url = format!("{}/daemon/agents/register", self.config.server_url);
        let heartbeat_url = format!("{}/daemon/agents/heartbeat", self.config.server_url);

        // Initial registration
        let mut request = client.post(&register_url)
            .json(&json!({
                "id": self.config.id,
                "name": self.config.name,
                "hostname": hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok()),
            }));

        if let Some(ref token) = self.config.token {
            request = request.header("token", token);
        }

        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("[Agent] Successfully registered with server");
                    self.status = AgentStatus::Online;
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    eprintln!("[Agent] Registration failed with status {}: {}", status, body);
                    return Err(anyhow!("Registration failed with status: {}", status));
                }
            }
            Err(e) => {
                eprintln!("[Agent] Registration request failed: {}", e);
                return Err(anyhow!(e));
            }
        }

        // Heartbeat loop
        loop {
            sleep(Duration::from_secs(self.config.heartbeat_interval)).await;

            let mut request = client.post(&heartbeat_url)
                .json(&json!({
                    "id": self.config.id,
                }));

            if let Some(ref token) = self.config.token {
                request = request.header("token", token);
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("[Agent] Heartbeat sent successfully");
                    } else {
                        let status = response.status();
                        eprintln!("[Agent] Heartbeat failed with status: {}", status);
                        return Err(anyhow!("Heartbeat failed with status: {}", status));
                    }
                }
                Err(e) => {
                    eprintln!("[Agent] Heartbeat request failed: {}", e);
                    return Err(anyhow!(e));
                }
            }
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
