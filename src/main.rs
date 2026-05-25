mod client;
mod ssh_server;
mod state;

use russh::{keys::ssh_key, server::Server};
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
        let key_str = std::fs::read(key_path)?;
        russh::keys::decode_openssh(&key_str, None)?
    } else {
        let new_key = russh::keys::PrivateKey::random(
            &mut russh::keys::key::safe_rng(),
            russh::keys::Algorithm::Ed25519,
        )?;
        let openssh_pem = new_key.to_openssh(ssh_key::LineEnding::LF)?;
        std::fs::write(key_path, &openssh_pem)?;
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

// let listener = TcpListener::bind("127.0.0.1:8080").await?;
// println!("Chat server listening on :8080");

// // watch channel for graceful shutdown
// let (shutdown_sender, shutdown_receiver) = watch::channel(false);

// loop {
//     tokio::select! {
//         accept_result = listener.accept() => {
//             let (socket, addr) = accept_result?;
//             println!("[{addr}] Connected");

//             let state_clone = _state.clone();
//             let shutdown_receiver_clone = shutdown_receiver.clone();
//             tokio::spawn(async move {
//                 client::handle_connection(socket, state_clone, shutdown_receiver_clone).await;
//                 println!("[{addr}] Disconnected");
//             });
//         }

//         _ = tokio::signal::ctrl_c() => {
//             println!("Shutting down server...");
//             shutdown_sender.send(true).unwrap();
//             break;
//         }
//     }
// }
