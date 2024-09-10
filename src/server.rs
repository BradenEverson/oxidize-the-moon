//! The server state implementation

use futures::stream::SplitSink;
use futures_util::{Future, StreamExt};
use http_body_util::Full;
use hyper::{
    body::{self, Bytes},
    service::Service,
    upgrade::Upgraded,
    Method, StatusCode,
};
use hyper::{Request, Response};
use std::{fs::File, io::Read, pin::Pin};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

/// A server service, responsible for sending the Websocket write stream over an mpsc channel
pub struct ServerService {
    /// The sender side of an mpsc channel that will handle all websocket write events
    pub sender: UnboundedSender<WebSocketWriteStream>,
}

/// An error from attempting to send over a tokio mpsc channel
pub type TokioMpscError = tokio::sync::mpsc::error::SendError<WebSocketWriteStream>;
/// A websocket write stream
pub type WebSocketWriteStream =
    SplitSink<WebSocketStream<hyper_util::rt::tokio::TokioIo<Upgraded>>, Message>;

impl ServerService {
    /// Creates a new service around a sender channel
    pub fn new(tx: UnboundedSender<WebSocketWriteStream>) -> Self {
        Self { sender: tx }
    }
}

impl Service<Request<body::Incoming>> for ServerService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<body::Incoming>) -> Self::Future {
        let tx = self.sender.clone();
        if hyper_tungstenite::is_upgrade_request(&req) {
            // Upgrade to WebSocket
            let (response, websocket) =
                hyper_tungstenite::upgrade(&mut req, None).expect("Error upgrading to WebSocket");
            tokio::spawn(async move {
                match websocket.await {
                    Ok(ws) => {
                        let (writer, mut reader) = ws.split();
                        tx.send(writer)?;

                        while let Some(Ok(msg)) = reader.next().await {
                            // TODO - Respond to websocket messages accordingly
                            match msg {
                                _ => {}
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to establish WebSocket Connection: {}", err)
                    }
                }
                Ok::<(), TokioMpscError>(())
            });

            Box::pin(async { Ok(response) })
        } else {
            // HTTP
            let response = Response::builder().status(StatusCode::OK);

            let res = match req.method() {
                &Method::GET => {
                    let path = match req.uri().path() {
                        "/" => "frontend/index.html",
                        _ => "frontend/404.html",
                    };

                    let page = File::open(path);
                    match page {
                        Ok(mut page) => {
                            let mut buf = vec![];
                            page.read_to_end(&mut buf).expect("Failed to read file");

                            response.body(Full::new(Bytes::copy_from_slice(&buf)))
                        }
                        Err(e) => {
                            panic!("{}{}", e, path);
                        }
                    }
                }
                _ => response.body(Full::new(Bytes::copy_from_slice(&[]))),
            };

            Box::pin(async { res })
        }
    }
}
