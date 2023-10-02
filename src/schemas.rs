use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::cfg::ClientCfg;

#[derive(Deserialize, Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct StatusResponse {
    pub valid: bool,
}

#[derive(Deserialize, Serialize)]
pub struct AddPeerSchema {
    #[serde(rename = "userIdentifier")]
    pub user_identifier: String,

    #[serde(rename = "peerIdentifier")]
    pub peer_identifier: String,

    #[serde(rename = "publicKey")]
    pub public_key: String,

    pub psk: Option<String>,

    #[serde(rename = "tunnelIpv4")]
    pub ipv4_tunnel_address: String,

    #[serde(rename = "tunnelIpv6")]
    pub ipv6_tunnel_address: String,
}

#[derive(Clone)]
pub struct ConfigState {
    pub(crate) config: Arc<ClientCfg>,
}
