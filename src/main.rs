mod client;
mod state;
mod ssh_server;

use state::SharedState;
use tokio::{net::TcpListener, sync::watch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _state = SharedState::new();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Chat server listening on :8080");

    // watch channel for graceful shutdown
    let (shutdown_sender, shutdown_receiver) = watch::channel(false);

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                let (socket, addr) = accept_result?;
                println!("[{addr}] Connected");

                let state_clone = _state.clone();
                let shutdown_receiver_clone = shutdown_receiver.clone();
                tokio::spawn(async move {
                    client::handle_connection(socket, state_clone, shutdown_receiver_clone).await;
                    println!("[{addr}] Disconnected");
                });
            }

            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down server...");
                shutdown_sender.send(true).unwrap();
                break;
            }
        }
    }

    Ok(())
}
