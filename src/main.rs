//! Basic Websocket Connection that can send metadata to all connected clients

use std::sync::Arc;

use futures::{executor, SinkExt};
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;

use oxidize_the_moon::server::{ServerService, WebSocketWriteStream};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let streams = Arc::new(Mutex::new(vec![]));

    let listener = TcpListener::bind("0.0.0.0:7878")
        .await
        .expect("Error starting up the server");

    let (tx, mut rx): (
        UnboundedSender<WebSocketWriteStream>,
        UnboundedReceiver<WebSocketWriteStream>,
    ) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        loop {
            let (socket, _) = listener
                .accept()
                .await
                .expect("Error accepting incoming connection");

            let io = TokioIo::new(socket);

            let server_service = ServerService::new(tx.clone());

            tokio::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, server_service)
                    .with_upgrades()
                    .await
                {
                    eprintln!("Error serving connection: {}", e);
                }
            });
        }
    });

    let streams_collector = streams.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            streams_collector.lock().await.push(msg)
        }
    });

    loop {
        streams.lock().await.iter_mut().for_each(|stream| {
            executor::block_on(stream.send(Message::text("Todo: Send actual data")))
                .expect("Failed to send message to client");
        });
    }
}
