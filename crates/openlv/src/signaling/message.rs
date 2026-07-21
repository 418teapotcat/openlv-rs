use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SignalingMessage {
    #[serde(rename = "flash")]
    Flash { payload: Value, timestamp: u64 },
    #[serde(rename = "pubkey")]
    Pubkey {
        payload: PubkeyPayload,
        timestamp: u64,
    },
    #[serde(rename = "capabilities")]
    Capabilities {
        payload: PeerCapabilities,
        timestamp: u64,
    },
    #[serde(rename = "data")]
    Data { payload: Value, timestamp: u64 },
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyPayload {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    pub identity: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct PeerCapabilities {
    pub transports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<PeerInfo>,
}

impl PeerInfo {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !is_bounded_string(&self.identity, 128) {
            return Err("peer identity must be 1-128 characters");
        }
        if !is_bounded_string(&self.name, 128) {
            return Err("peer name must be 1-128 characters");
        }
        if let Some(icon) = &self.icon
            && !is_bounded_string(icon, 8192)
        {
            return Err("peer icon must be 1-8192 characters");
        }
        Ok(())
    }
}

impl PeerCapabilities {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.transports.is_empty() || self.transports.len() > 8 {
            return Err("transports must contain 1-8 entries");
        }
        if !self
            .transports
            .iter()
            .all(|transport| is_bounded_string(transport, 32))
        {
            return Err("transport identifiers must be 1-32 characters");
        }
        if let Some(info) = &self.info {
            info.validate()?;
        }
        Ok(())
    }
}

impl SignalingMessage {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::Flash { payload, .. } | Self::Data { payload, .. } if !payload.is_object() => {
                Err("signaling payload must be an object")
            }
            Self::Pubkey { payload, .. } if payload.public_key.is_empty() => {
                Err("public key must not be empty")
            }
            Self::Capabilities { payload, .. } => payload.validate(),
            _ => Ok(()),
        }
    }
}

fn is_bounded_string(value: &str, max_length: usize) -> bool {
    !value.is_empty() && value.len() <= max_length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_the_v0_1_capabilities_wire_shape() {
        let message = SignalingMessage::Capabilities {
            payload: PeerCapabilities {
                transports: vec!["wrtc".into(), "ws".into()],
                info: Some(PeerInfo {
                    identity: "com.example.wallet".into(),
                    name: "Example Wallet".into(),
                    icon: Some("https://example.com/icon.png".into()),
                }),
            },
            timestamp: 1_750_000_000_000,
        };

        assert_eq!(
            serde_json::to_value(message).unwrap(),
            serde_json::json!({
                "type": "capabilities",
                "payload": {
                    "transports": ["wrtc", "ws"],
                    "info": {
                        "identity": "com.example.wallet",
                        "name": "Example Wallet",
                        "icon": "https://example.com/icon.png"
                    }
                },
                "timestamp": 1_750_000_000_000u64
            })
        );
    }

    #[test]
    fn validates_capabilities_bounds() {
        let valid = PeerCapabilities {
            transports: vec!["wrtc".into()],
            info: Some(PeerInfo {
                identity: "com.example.wallet".into(),
                name: "Example Wallet".into(),
                icon: None,
            }),
        };
        assert!(valid.validate().is_ok());

        assert!(
            PeerCapabilities {
                transports: vec![],
                info: None,
            }
            .validate()
            .is_err()
        );
        assert!(
            PeerInfo {
                identity: "".into(),
                name: "Example Wallet".into(),
                icon: None,
            }
            .validate()
            .is_err()
        );
        assert!(
            PeerInfo {
                identity: "com.example.wallet".into(),
                name: "Example Wallet".into(),
                icon: Some("x".repeat(8193)),
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn rejects_the_retired_handshake_ack_packet() {
        assert!(
            serde_json::from_value::<SignalingMessage>(serde_json::json!({
                "type": "ack",
                "timestamp": 1,
                "payload": null,
            }))
            .is_err()
        );
    }
}
