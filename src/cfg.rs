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
        env = "WPM_DNS",
        long = "dns",
        default_value = "",
    )]
    pub(crate) dns: String,

    #[arg(
        env = "WPM_ENDPOINT",
        long = "endpoint",
        default_value = "",
        // TODO default value: public ipv4 address (e.g. curl ifconfig.io -4)
    )]
    pub(crate) endpoint: String,

    #[arg(
        env = "WPM_INTERFACE",
        long = "interface",
        default_value = "wg100",
        value_enum,
        ignore_case(true)
    )]
    pub(crate) interface: String,
}
