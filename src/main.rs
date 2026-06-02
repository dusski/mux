mod ssh_server;
mod state;

use russh::{server::Server};
use ssh_server::ChatSSHServer;
use state::SharedState;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = SharedState::new();

    let mut config = russh::server::Config::default();
    config.limits.rekey_time_limit = std::time::Duration::from_secs(10);

    let key_path = std::path::Path::new("host_key.pem");
    let host_key = if key_path.exists() {
        let key_str = std::fs::read_to_string(key_path)?;
        russh::keys::load_secret_key(&key_str, None)?
    } else {
        let new_key = russh::keys::PrivateKey::random(
            &mut russh::keys::key::safe_rng(),
            russh::keys::Algorithm::Ed25519,
        )?;
        let openssh_key_string = new_key.to_openssh(russh::keys::ssh_key::LineEnding::LF)?;
        std::fs::write(key_path, openssh_key_string.as_bytes())?;
        println!("Generated new host key and saved to {:?}", key_path);
        new_key
    };

    // let keypair: russh::keys::PrivateKey =
    config.keys.push(host_key);

    let config = Arc::new(config);
    let mut server = ChatSSHServer { state };

    println!("SSH Chat server listening on 127.0.0.1:2222");

    server.run_on_address(config, ("127.0.0.1", 2222)).await?;

    Ok(())
}
