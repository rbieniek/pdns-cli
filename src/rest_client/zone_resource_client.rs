use tokio::sync::oneshot::{channel, Receiver, Sender};

use crate::pdns::common::PowerDnsPayload;
use crate::pdns::zone::Zone;
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::rest_client::errors::RestClientError;
use crate::rest_client::pdns_resource_client::{PathProvider, PnsServerResponse, PowerDnsRestClient, RequestEvent, RequestPathProvider, ResponseEvent};

pub struct ZoneResourceClient {
    pdns_resource_client: PowerDnsRestClient,
}

pub struct GetZoneRequestEvent {
    zone_name: String,
}

pub struct GetZoneResponseEvent {
    result: Result<Zone, RestClientError>,
}

struct GetZoneRequestPathProvider {
    request_path: String,
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
            .handle_request_event_alt::<GetZoneRequestEvent, Zone, PathProvider<GetZoneRequestEvent>>(request_rx,
                                      response_tx,
                                      request_path);
    }
}

impl GetZoneResponseEvent {
    fn new(result: Result<Zone, RestClientError>) -> GetZoneResponseEvent {
        GetZoneResponseEvent {
            result,
        }
    }
}

impl ResponseEvent<Zone> for GetZoneResponseEvent {
    fn result(&self) -> Result<Zone, RestClientError> {
        self.result.clone()
    }
}

impl RequestEvent<Zone> for GetZoneRequestEvent {
    fn response(&self, result: Result<Zone, RestClientError>) -> Box<dyn ResponseEvent<Zone>> {
        Box::new(GetZoneResponseEvent::new(result))
    }

    fn request_path_provider(&self) -> Box<dyn RequestPathProvider> {
        Box::new(GetZoneRequestPathProvider::new(&self.zone_name))
    }
}

impl GetZoneRequestPathProvider {
    fn new(zone_name: &String) -> GetZoneRequestPathProvider {
        return GetZoneRequestPathProvider {
            request_path: format!("zones/{}", zone_name)
        };
    }
}

impl RequestPathProvider for GetZoneRequestPathProvider {
    fn provide_request_path(&self) -> String {
        self.request_path.clone()
    }
}

fn request_path(request: &GetZoneRequestEvent) -> String {
    format!("zones/{}", &request.zone_name)
}