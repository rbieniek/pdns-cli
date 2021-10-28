use crate::pdns::server::Server;
use crate::rest_client::errors::RestClientError;
use tokio::sync::mpsc::Receiver;
use crate::rest_client::lifecycle::Disposeable;
use tokio::task::JoinHandle;

pub struct GetServerRequestEvent {
    base_uri: String,
    response_channel: Receiver<GetServerResponseEvent>,
}

pub struct GetServerResponseEvent {
    result: Result<Server, RestClientError>
}

pub struct ServerResourceClient {
    join_handles: Vec<JoinHandle<()>>,
}

impl ServerResourceClient {
    pub fn new() -> ServerResourceClient {
        ServerResourceClient {
            join_handles: Vec::new(),
        }
    }
}

impl Disposeable for ServerResourceClient {
    fn shutdown(&self) {
        for handle in self.join_handles.iter() {
            handle.abort();
        }
    }
}