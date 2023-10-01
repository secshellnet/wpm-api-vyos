use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
#[clap(version)]
pub(crate) struct ClientCfg {
  #[arg(env = "WPM_LISTEN_ADDR", long = "listen_addr", default_value = "0.0.0.0:8080")]
  pub(crate) listen_addr: SocketAddr,

  #[arg(env = "WPM_SECRET")]
  pub(crate) secret: String,

  #[arg(env = "WPM_INTERFACE", default_value = "wg100", value_enum, ignore_case(true))]
  pub(crate) interface: String,
}
