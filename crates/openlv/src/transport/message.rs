use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct TransportNegotiationMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub payload: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum SessionMessage {
    #[serde(rename = "request")]
    Request {
        #[serde(rename = "messageId")]
        message_id: String,
        payload: Value,
    },
    #[serde(rename = "response")]
    Response {
        #[serde(rename = "messageId")]
        message_id: String,
        payload: Value,
    },
    #[serde(rename = "ack")]
    Ack {
        #[serde(rename = "messageId")]
        message_id: String,
    },
}
