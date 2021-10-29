use log::info;
use tokio::sync::oneshot::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::pdns::server::Server;
use crate::rest_client::errors::RestClientError;
use crate::rest_client::lifecycle::Disposeable;

pub struct GetServerRequestEvent {
    base_uri: String,
    response_channel: Sender<GetServerResponseEvent>,
}

pub struct GetServerResponseEvent {
    result: Result<Server, RestClientError>,
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
        let (event_tx, event_rx) = channel::<GetServerRequestEvent>();

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
    match event_rx.await {
        Ok(message) => {
            info!("Received GetServerRequestEvent(baseUri={})", &message.base_uri);
        }
        Err(error) => info!("Expected message, didn't get one, error {}", error.to_string())
    }
}