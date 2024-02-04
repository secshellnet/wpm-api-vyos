use std::net::IpAddr;

use axum::{
    extract::{Json, Path, State},
    http::{Response, StatusCode},
};
use serde::Serialize;
use tracing::{error, info};

use crate::schemas::{NewPeerRequest, ApiResponse, PeerResponse, ConfigState, IsValidResponse, IdentifierListResponse};
use crate::utils::{apply_vyatta_cfg, validate_identifier, validate_key, validate_name, validate_device_id};

#[derive(Serialize)]
pub enum ApiReturnTypes {
    ApiResponse(ApiResponse),
    IsValidResponse(IsValidResponse),
    PeerResponse(PeerResponse),
    ListOfApiResponses(Vec<ApiResponse>),
    IdentifierListResponse(IdentifierListResponse),
}

pub async fn get_peers() -> Result<Json<ApiReturnTypes>, StatusCode> {
    info!("? all");
    
    // TODO get identifier or peers

    let response_data = IdentifierListResponse { peers: [].to_vec() };

    Ok(Json(ApiReturnTypes::IdentifierListResponse(response_data)))
}

pub async fn get_peer(
    State(config): State<ConfigState>,
    Path(identifier): Path<String>,
) -> Result<Json<ApiReturnTypes>, StatusCode> {
    if !validate_identifier(&identifier) {
        return Err(StatusCode::BAD_REQUEST);
    }

    info!("? {}", identifier);

    /*// get existing allowed ips for this identifier from current vyatta configuration
    // vyatta config systems used transactions, so if one setting is there everything is there
    let vyatta_config = format!(
        "\
        source /opt/vyatta/etc/functions/script-template\n\
        run show configuration commands | match 'interfaces wireguard {} peer {}'\n\
        exit",
        config.config.interface, identifier
    );

    let stdout = apply_vyatta_cfg(vyatta_config).await.map_err(|err| {
        error!("Unable to apply vyatta cfg: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // valid means is configured
    // when a peer has been marked for deletion
    //   valid=false indicates that a peer has been deleted
    // when a peer have just been created:
    //   valid=true indicates that the peer has been created successfully
    let valid = stdout.len() > 1;*/

    let response_data = ApiResponse { success: false, message: String::new() };

    Ok(Json(ApiReturnTypes::ApiResponse(response_data)))
}

pub async fn add_peer(
    State(config): State<ConfigState>,
    Json(peer_data): Json<NewPeerRequest>,
) -> Result<Json<ApiReturnTypes>, StatusCode> {
    let mut response_data = vec![];

    // check if parameter values are valid
    if let Err(_) = validate_name(&peer_data.firstname.to_string()) {
        response_data.push(ApiResponse {
            success: false,
            message: String::from("Invalid value for parameter firstname"),
        });
    }

    if let Err(_) = validate_name(&peer_data.lastname.to_string()) {
        response_data.push(ApiResponse {
            success: false,
            message: String::from("Invalid value for parameter lastname"),
        });
    }

    if let Err(_) = validate_device_id(&peer_data.device_id.to_string()) {
        response_data.push(ApiResponse {
            success: false,
            message: String::from("Invalid value for parameter deviceId"),
        });
    }

    if let Err(_) = validate_key(&peer_data.public_key.to_string()) {
        response_data.push(ApiResponse {
            success: false,
            message: String::from("Invalid value for parameter publicKey"),
        });
    }

    if let Some(psk) = &peer_data.psk {
        if let Err(_) = validate_key(&psk) {
            response_data.push(ApiResponse {
                success: false,
                message: String::from("Invalid value for parameter psk"),
            });
        }
    }

    let identifier = format!(
        "{}-{}-{}",
        peer_data.firstname, peer_data.lastname, peer_data.device_id
    );
    info!("+ {}", identifier);

    if response_data.len() > 0 {
        // TODO send http 400 bad request
        return Ok(Json(ApiReturnTypes::ListOfApiResponses(response_data)));
    }

    // TODO generate next free addresses

    /*// build the commands to reconfigure VyOS
    let mut commands = format!(
        "\
        set firewall group address-group VPN-{firstname}-{lastname} address '{address4}'\n\
        set firewall group ipv6-address-group VPN-{firstname}-{lastname}-6 address '{address6}'\n\
        set interfaces wireguard {interface} peer {firstname}-{lastname}-{device_id} allowed-ips '{address4}/32'\n\
        set interfaces wireguard {interface} peer {firstname}-{lastname}-{device_id} allowed-ips '{address6}/128'\n\
        set interfaces wireguard {interface} peer {firstname}-{lastname}-{device_id} pubkey '{public_key}'",
        user_identifier = peer_data.user_identifier,
        address4 = peer_data.ipv4_tunnel_address,
        address6 = peer_data.ipv6_tunnel_address,
        firstname = peer_data.firstname,
        lastname = peer_data.lastname,
        device_id = peer_data.device_id,
        interface = config.config.interface,
        public_key = peer_data.public_key,
    );

    // in case a psk is given, add it to the configuration
    if let Some(psk) = &peer_data.psk {
        let psk_config = format!(
            "set interfaces wireguard {interface} peer {firstname}-{lastname}-{device_id} preshared-key '{psk}';",
            interface = config.config.interface,
            firstname = peer_data.firstname,
            lastname = peer_data.lastname,
            device_id = peer_data.device_id,
            psk = psk
        );
        commands = format!("{}\n{}", commands, psk_config);
    }

    // wrap the commands with the commands necessary to enter and exit configuration mode
    let vyatta_config = format!(
        "\
        source /opt/vyatta/etc/functions/script-template\n\
        {}\n\
        commit;save;exit",
        commands
    );

    tokio::spawn(async move {
        if let Err(err) = apply_vyatta_cfg(vyatta_config).await {
            error!("Failed to apply vyatta config: {:?}", err);
        }
    });*/

    Ok(Json(ApiReturnTypes::ListOfApiResponses(response_data)))
}

pub async fn delete_peer(
    State(config): State<ConfigState>,
    Path(identifier): Path<String>,
) -> Result<Json<ApiReturnTypes>, StatusCode> {
    if !validate_identifier(&identifier) {
        return Err(StatusCode::BAD_REQUEST);
    }
    info!("- {}", identifier);

    /*// get existing allowed ips for this identifier from current vyatta configuration
    let vyatta_config = format!(
        "\
        source /opt/vyatta/etc/functions/script-template\n\
        run show configuration commands | match 'interfaces wireguard {} peer {} allowed-ips'\n\
        exit",
        config.config.interface, identifier
    );

    let stdout = apply_vyatta_cfg(vyatta_config).await.map_err(|err| {
        error!("Unable to apply vyatta cfg: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let lines = stdout.split(|&x| x == b'\n');

    let mut address4 = String::from("");
    let mut address6 = String::from("");
    for line in lines {
        let line_str = String::from_utf8_lossy(line);
        let splitted: Vec<&str> = line_str.trim().split_whitespace().collect();

        if splitted.len() >= 8 {
            // remove single quotes
            let mut address = splitted[7]
                .strip_prefix("'")
                .unwrap()
                .strip_suffix("'")
                .unwrap();

            // remove netmask
            address = address.split('/').next().expect("No parts found");

            //println!("{}", address);

            // determine ipv4 / ipv6 address
            match address.parse::<IpAddr>() {
                Ok(ip_addr) => match ip_addr {
                    IpAddr::V4(_) => {
                        address4 = address.to_string();
                    }
                    IpAddr::V6(_) => {
                        address6 = address.to_string();
                    }
                },
                Err(_) => error!("Invalid IP address format"),
            }
            //println!("{}", address4);
            //println!("{}", address6);
        }
    }

    let identifier_parts: Vec<&str> = identifier.split('-').collect();
    let user_identifier = format!("{}-{}", identifier_parts[0], identifier_parts[0]);

    let vyatta_config = format!(
        "\
        source /opt/vyatta/etc/functions/script-template\n\
        delete interface wireguard {interface} peer {identifier}\n\
        delete firewall group address-group VPN-{user_identifier} address {address4}
        delete firewall group ipv6-address-group VPN-{user_identifier}-6 address {address6}
        commit;save;exit",
        interface = config.config.interface,
        identifier = identifier,
        user_identifier = user_identifier,
        address4 = address4,
        address6 = address6,
    );

    tokio::spawn(async move {
        if let Err(err) = apply_vyatta_cfg(vyatta_config).await {
            error!("Failed to apply vyatta config: {:?}", err);
        }
    });*/

    let response_data = ApiResponse { success: false, message: String::new() };

    Ok(Json(ApiReturnTypes::ApiResponse(response_data)))
}
