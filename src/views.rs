use std::net::IpAddr;

use axum::{
    extract::{Json, Path, State},
    http::{Response, StatusCode},
};
use serde::Serialize;
use tracing::{error, info};

use crate::schemas::{AddPeerSchema, ApiResponse, ConfigState, StatusResponse};
use crate::utils::{apply_vyatta_cfg, validate_identifier, validate_key};

#[derive(Serialize)]
pub enum ApiReturnTypes {
    ApiResponse(ApiResponse),
    StatusResponse(StatusResponse),
    ListOfApiResponses(Vec<ApiResponse>),
}

pub async fn wpm_redirect() -> Response<String> {
    let location = "https://wpm.general.pve3.secshell.net";
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", location)
        .body(format!("Redirecting to {}", location))
        .unwrap()
}

pub async fn get_peer(
    State(config): State<ConfigState>,
    Path(identifier): Path<String>,
) -> Result<Json<ApiReturnTypes>, StatusCode> {
    validate_identifier(&identifier).map_err(||StatusCode::BAD_REQUEST)?;

    info!("? {}", identifier);

    // get existing allowed ips for this identifier from current vyatta configuration
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
    let valid = stdout.len() > 1;

    let response_data = StatusResponse { valid };

    Ok(Json(ApiReturnTypes::StatusResponse(response_data)))
}

pub async fn add_peer(
    State(config): State<ConfigState>,
    Json(peer_data): Json<AddPeerSchema>,
) -> Result<Json<ApiReturnTypes>, StatusCode> {
    let mut response_data = vec![];

    // check if the public key is valid
    if let Err(_) = validate_key(&peer_data.public_key.to_string()) {
        response_data.push(ApiResponse {
            status: String::from("error"),
            message: String::from("invalid value for parameter publicKey"),
        });
    }

    // check if the pre-shared key is valid
    if let Some(psk) = &peer_data.psk {
        if let Err(_) = validate_key(&psk) {
            response_data.push(ApiResponse {
                status: String::from("error"),
                message: String::from("invalid value for parameter psk"),
            });
        }
    }

    // todo sanitize addresses, user identifier, peer identifier

    // generate the identifier, which is the user identifier + peer identifier
    let identifier = format!("{}-{}", peer_data.user_identifier, peer_data.peer_identifier);
    info!("+ {}", identifier);
    validate_identifier(&identifier).map_err(||StatusCode::BAD_REQUEST)?;

    // VyOS allows the label for a peer to have 100 characters.
    if identifier.len() > 100 {
        response_data.push(ApiResponse {
            status: String::from("error"),
            message: String::from("invalid value for parameter identifier"),
        });
    }

    if response_data.len() > 0 {
        // TODO send bad request, eg. err
        return Ok(Json(ApiReturnTypes::ListOfApiResponses(response_data)));
    }

    // build the commands to reconfigure VyOS
    let mut commands = format!(
        "\
        set firewall group address-group VPN-{user_identifier} address '{address4}'\n\
        set firewall group ipv6-address-group VPN-{user_identifier}-6 address '{address6}'\n\
        set interfaces wireguard {interface} peer {identifier} allowed-ips '{address4}/32'\n\
        set interfaces wireguard {interface} peer {identifier} allowed-ips '{address6}/128'\n\
        set interfaces wireguard {interface} peer {identifier} pubkey '{public_key}'",
        user_identifier = peer_data.user_identifier,
        address4 = peer_data.ipv4_tunnel_address,
        address6 = peer_data.ipv6_tunnel_address,
        identifier = identifier,
        interface = config.config.interface,
        public_key = peer_data.public_key,
    );

    // in case a psk is given, add it to the configuration
    if let Some(psk) = &peer_data.psk {
        let psk_config = format!(
            "set interfaces wireguard {interface} peer {identifier} preshared-key '{psk}';",
            interface = config.config.interface,
            identifier = identifier,
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
    });

    let response_data = ApiResponse {
        status: String::from("success"),
        message: format!("peer {} will be created", identifier),
    };

    Ok(Json(ApiReturnTypes::ApiResponse(response_data)))
}

pub async fn delete_peer(
    State(config): State<ConfigState>,
    Path(identifier): Path<String>,
) -> Result<Json<ApiReturnTypes>, StatusCode> {
    validate_identifier(&identifier).map_err(||StatusCode::BAD_REQUEST)?;
    info!("- {}", identifier);

    // get existing allowed ips for this identifier from current vyatta configuration
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
    });

    let response_data = ApiResponse {
        status: String::from("success"),
        message: format!("peer {} will be deleted", identifier),
    };

    Ok(Json(ApiReturnTypes::ApiResponse(response_data)))
}
