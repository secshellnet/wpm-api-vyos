use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
#[clap(version)]
pub(crate) struct ClientCfg {
    #[arg(
        env = "WPM_LISTEN_ADDR",
        long = "listen_addr",
        default_value = "0.0.0.0:8002"
    )]
    pub(crate) listen_addr: SocketAddr,

    #[arg(
        env = "WPM_SECRET",
        long = "secret",
    )]
    pub(crate) secret: String,

    #[arg(
        env = "WPM_SERVER_ID",
        long = "server_id",
        default_value = "",
        // TODO default value: gethostname::gethostname()
    )]
    pub(crate) server_id: String,

    #[arg(
        env = "WPM_DNS4",
        long = "dns4",
        default_value = "",
    )]
    pub(crate) dns4: String,

    #[arg(
        env = "WPM_DNS6",
        long = "dns6",
        default_value = "",
    )]
    pub(crate) dns6: String,

    #[arg(
        env = "WPM_ENDPOINT4",
        long = "endpoint4",
        default_value = "",
        // TODO default value: public ipv4 address (e.g. curl ifconfig.io -4)
    )]
    pub(crate) endpoint4: String,

    #[arg(
        env = "WPM_ENDPOINT6",
        long = "endpoint6",
        default_value = "",
        // TODO default value: public ipv6 address (e.g. curl ifconfig.io -6)
    )]
    pub(crate) endpoint6: String,

    #[arg(
        env = "WPM_INTERFACE",
        long = "interface",
        default_value = "wg100",
        value_enum,
        ignore_case(true)
    )]
    pub(crate) interface: String,
}
