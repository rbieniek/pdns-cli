use std::sync::Arc;

use tokio::sync::oneshot::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::pdns::zone::Zone;
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::rest_client::pdns_resource_client::{PathProvider, PnsServerResponse, PowerDnsRestClient};

pub struct ZoneResourceClient {
    pdns_resource_client: Arc<PowerDnsRestClient>,
    join_handles: Vec<JoinHandle<()>>,
}

pub struct GetZoneRequestEvent {
    zone_name: String,
}

impl ZoneResourceClient {
    pub fn new(base_uri: &String, api_key: &String) -> ZoneResourceClient {
        ZoneResourceClient {
            pdns_resource_client: Arc::new(PowerDnsRestClient::new(ClientRequestBuilder::new(base_uri, api_key))),
            join_handles: Vec::new(),
        }
    }

    pub fn spawn_handler(&mut self,
                         request_rx: Receiver<GetZoneRequestEvent>,
                         response_tx: Sender<PnsServerResponse<GetZoneRequestEvent, Zone>>) {
        self.join_handles.push(tokio::spawn(handle_request_event(self.pdns_resource_client.clone(),
                                                                 request_rx,
                                                                 response_tx)));
    }
}

impl GetZoneRequestEvent {
    pub fn new(zone_name: &String) -> GetZoneRequestEvent {
        GetZoneRequestEvent {
            zone_name: zone_name.clone(),
        }
    }
}

impl Drop for ZoneResourceClient {
    fn drop(&mut self) {
        for handle in self.join_handles.iter() {
            handle.abort();
        }
    }
}

async fn handle_request_event(pdns_resource_client: Arc<PowerDnsRestClient>,
                              request_rx: Receiver<GetZoneRequestEvent>,
                              response_tx: Sender<PnsServerResponse<GetZoneRequestEvent, Zone>>) {
    pdns_resource_client
        .handle_request_event::<GetZoneRequestEvent,
            Zone,
            PathProvider<GetZoneRequestEvent>>(request_rx,
                                               response_tx,
                                               request_path).await
}

fn request_path(request: &GetZoneRequestEvent) -> String {
    format!("servers/localhost/zones/{}", &request.zone_name)
}