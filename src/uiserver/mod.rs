
use std::{
  sync::{mpsc::Receiver, Arc}
};

use crate::plugin_state::{PluginState, StateUpdate};

use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

pub(super) struct UiServer {
  remote_state: Arc<PluginState>,
  incoming: Receiver<StateUpdate>,
}

impl UiServer {
  pub fn new(remote_state: Arc<PluginState>, incoming: Receiver<StateUpdate>) -> Self {

    let addr = "127.0.0.1:8080".to_string();
    let addr = addr.parse::<SocketAddr>()?;

    let listener = TcpListener::bind(&addr)?;

      Self {
          remote_state,
          incoming,
      }
  }
}