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
use std::{fs::File, io::Read, pin::Pin, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::data::{handler::DataHandler, Data};

/// A server service, responsible for sending the Websocket write stream over an mpsc channel
pub struct ServerService<HANDLER>
where
    HANDLER: DataHandler + Send + Sync + 'static,
{
    /// The sender side of an mpsc channel that will handle all websocket write events
    pub sender: UnboundedSender<WebSocketWriteStream>,
    /// The dynamic data handler for any type of handleable data
    pub handler: Arc<HANDLER>,
}

/// An error from attempting to send over a tokio mpsc channel
pub type TokioMpscError = tokio::sync::mpsc::error::SendError<WebSocketWriteStream>;
/// A websocket write stream
pub type WebSocketWriteStream =
    SplitSink<WebSocketStream<hyper_util::rt::tokio::TokioIo<Upgraded>>, Message>;

impl<HANDLER> ServerService<HANDLER>
where
    HANDLER: DataHandler + Send + Sync + 'static,
{
    /// Creates a new service around a sender channel
    pub fn new(tx: UnboundedSender<WebSocketWriteStream>, handler: HANDLER) -> Self {
        Self {
            sender: tx,
            handler: Arc::new(handler),
        }
    }
}

impl<HANDLER> Service<Request<body::Incoming>> for ServerService<HANDLER>
where
    HANDLER: DataHandler + Send + Sync + 'static,
{
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<body::Incoming>) -> Self::Future {
        let tx = self.sender.clone();
        if hyper_tungstenite::is_upgrade_request(&req) {
            // Upgrade to WebSocket
            let (response, websocket) =
                hyper_tungstenite::upgrade(&mut req, None).expect("Error upgrading to WebSocket");
            let handler = self.handler.clone();
            tokio::spawn(async move {
                match websocket.await {
                    Ok(ws) => {
                        let (writer, mut reader) = ws.split();
                        tx.send(writer)?;

                        while let Some(Ok(msg)) = reader.next().await {
                            match msg {
                                Message::Text(serialized_string) => {
                                    let handler_clone = handler.clone();
                                    tokio::spawn(async move {
                                        let handleable_data: Data =
                                            serde_json::from_str(&serialized_string)
                                                .expect("Failed to parse data as an expected type");
                                        handler_clone.handle(handleable_data)
                                    });
                                }
                                Message::Binary(_controller) => {
                                    tokio::spawn(async move {});
                                }
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
            let mut response = Response::builder().status(StatusCode::OK);

            let res = match req.method() {
                &Method::GET => {
                    let path = match req.uri().path() {
                        "/" => "frontend/index.html",
                        "/dist/data-collection.js" => {
                            response = response.header("Content-Type", "application/javascript");

                            "frontend/dist/data-collection.js"
                        }
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
