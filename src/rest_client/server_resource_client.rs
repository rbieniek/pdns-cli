use std::sync::Arc;

use tokio::sync::oneshot::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::pdns::server::Server;
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::rest_client::pdns_resource_client::{PnsServerResponse, PowerDnsRestClient};

pub struct ServerResourceClient {
    pdns_resource_client: Arc<PowerDnsRestClient>,
    join_handles: Vec<JoinHandle<()>>,
}

pub struct QueryServerRequestEvent {}

impl QueryServerRequestEvent {
    pub fn new() -> QueryServerRequestEvent {
        QueryServerRequestEvent {}
    }
}

impl ServerResourceClient {
    pub fn new(base_uri: &String, api_key: &String) -> ServerResourceClient {
        ServerResourceClient {
            pdns_resource_client: Arc::new(PowerDnsRestClient::new(ClientRequestBuilder::new(base_uri, api_key))),
            join_handles: Vec::new(),
        }
    }

    pub fn spawn_query(&mut self,
                       request_rx: Receiver<QueryServerRequestEvent>,
                       response_tx: Sender<PnsServerResponse<QueryServerRequestEvent, Server>>) {
        self.join_handles.push(tokio::spawn(handle_get_request(self.pdns_resource_client.clone(),
                                                               request_rx,
                                                               response_tx)));
    }
}

impl Drop for ServerResourceClient {
    fn drop(&mut self) {
        for handle in self.join_handles.iter() {
            handle.abort();
        }
    }
}

async fn handle_get_request(pdns_resource_client: Arc<PowerDnsRestClient>,
                            request_rx: Receiver<QueryServerRequestEvent>,
                            response_tx: Sender<PnsServerResponse<QueryServerRequestEvent, Server>>) {
    pdns_resource_client
        .handle_get_request::<QueryServerRequestEvent, Server>(request_rx,
                                                               response_tx,
                                                               get_server_request_path).await
}

fn get_server_request_path(_request: &QueryServerRequestEvent) -> String {
    "servers/localhost".to_string()
}
