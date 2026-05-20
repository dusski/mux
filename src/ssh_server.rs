use russh::server::{Auth, Msg, Session};
use russh::{Channel, ChannelId};
use async_trait::async_trait;
use crate::state::SharedState;

#[derive(Clone)]
pub struct ChatSSHServer {
  pub state: SharedState,
}

impl russh::server::Server for ChatSSHServer {
  type Handler = ClientSession;

  fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
    ClientSession {
        state: self.state.clone(),
    }
  }
}

pub struct ClientSession {
  pub state: SharedState,
}

impl russh::server::Handler for ClientSession {
  type Error = anyhow::Error;

  async fn auth_publickey(
    &mut self,
    _user: &str,
    _public_key: &russh::keys::PublicKey,
  ) -> Result<Auth, Self::Error> {
    Ok(Auth::Accept)
  }

  async fn channel_open_session(
    &mut self,
    _channel: Channel<Msg>,
    _session: &mut Session,
  ) -> Result<bool, Self::Error> {
    Ok(true)
  }

  async fn data(
    &mut self,
    _channel: ChannelId,
    _data: &[u8],
    _session: &mut Session,
  ) -> Result<(), Self::Error> {
    Ok(())
  }
}