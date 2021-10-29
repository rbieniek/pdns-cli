use crate::pdns::server::Server;
use crate::rest_client::errors::RestClientError;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::rest_client::lifecycle::Disposeable;
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use std::env::join_paths;
use log::info;

pub struct GetServerRequestEvent {
    base_uri: String,
    response_channel: Sender<GetServerResponseEvent>,
}

pub struct GetServerResponseEvent {
    result: Result<Server, RestClientError>
}

pub struct ServerResourceClient {
    join_handles: Vec<JoinHandle<()>>,
}

impl GetServerRequestEvent {
    pub fn new(base_uri: String,
               response_channel: Sender<GetServerResponseEvent>) -> GetServerRequestEvent {
        GetServerRequestEvent {
            base_uri,
            response_channel,
        }
    }
}

impl ServerResourceClient {
    pub fn new() -> ServerResourceClient {
        ServerResourceClient {
            join_handles: Vec::new(),
        }
    }

    pub fn create_channel(&mut self) -> Sender<GetServerRequestEvent> {
        let (event_tx, event_rx) = mpsc::channel::<GetServerRequestEvent>(32);

        self.join_handles.push(tokio::spawn(handle_request_event(event_rx)));

        return event_tx;
    }
}

impl Disposeable for ServerResourceClient {
    fn shutdown(&self) {
        for handle in self.join_handles.iter() {
            handle.abort();
        }
    }
}

async fn handle_request_event(mut event_rx: Receiver<GetServerRequestEvent>) {
    while let Some(message) = event_rx.recv().await {
        info!("Received GetServerRequestEvent(baseUri={})", &message.base_uri);


    }
}