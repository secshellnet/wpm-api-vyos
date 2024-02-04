use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::cfg::ClientCfg;

#[derive(Deserialize, Serialize)]
pub struct IsValidResponse {
    pub valid: bool,
}

#[derive(Deserialize, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct NewPeerRequest {
    pub firstname: String,
    pub lastname: String,

    #[serde(rename = "deviceId")]
    pub device_id: String,

    #[serde(rename = "publicKey")]
    pub public_key: String,

    pub psk: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct PeerResponse {
    pub firstname: String,
    pub lastname: String,

    #[serde(rename = "deviceId")]
    pub device_id: String,

    #[serde(rename = "serverId")]
    pub server_id: String,

    #[serde(rename = "publicKey")]
    pub public_key: String,

    // TODO fix datatype
    pub endpoint: Vec<String>,

    pub psk: Option<String>,

    #[serde(rename = "tunnelAddress")]
    // TODO fix datatype
    pub tunnel_addr: Vec<String>,

    // TODO fix datatype
    pub dns: Option<Vec<String>>,

    pub valid: bool,
}

#[derive(Deserialize, Serialize)]
pub struct IdentifierListResponse {
    pub peers: Vec<String>,
} 

#[derive(Clone)]
pub struct ConfigState {
    pub(crate) config: Arc<ClientCfg>,
}
