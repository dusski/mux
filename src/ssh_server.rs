use crate::state::SharedState;
use async_trait::async_trait;
use russh::server::{Auth, Msg, Session};
use russh::{Channel, ChannelId};

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
        println!("A client successfully opened an SSH terminal channel.");

        let welcome_message = "Welcome to the Async SSH Chat Room!\r\nType something: ";

        _session.data(_channel.id(), welcome_message.as_bytes())?;
        Ok(true)
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        _data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        println!("Server received raw bytes from client: {:?}", _data);

        _session.data(_channel, _data.to_vec())?;

        Ok(())
    }
}
