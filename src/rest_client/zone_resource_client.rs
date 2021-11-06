use tokio::sync::oneshot::{Receiver, Sender};

use crate::pdns::zone::Zone;
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::rest_client::pdns_resource_client::{PathProvider, PnsServerResponse, PowerDnsRestClient};

pub struct ZoneResourceClient {
    pdns_resource_client: PowerDnsRestClient,
}

pub struct GetZoneRequestEvent {
    zone_name: String,
}

impl ZoneResourceClient {
    pub fn new(base_uri: &String, api_key: &String) -> ZoneResourceClient {
        ZoneResourceClient {
            pdns_resource_client: PowerDnsRestClient::new(ClientRequestBuilder::new(base_uri, api_key))
        }
    }

    pub async fn handle_request_event(&self,
                                      request_rx: Receiver<GetZoneRequestEvent>,
                                      response_tx: Sender<PnsServerResponse<GetZoneRequestEvent, Zone>>) {
        &self.pdns_resource_client
            .handle_request_event::<GetZoneRequestEvent,
                Zone,
                PathProvider<GetZoneRequestEvent>>(request_rx,
                                                   response_tx,
                                                   request_path);
    }
}

fn request_path(request: &GetZoneRequestEvent) -> String {
    format!("zones/{}", &request.zone_name)
}