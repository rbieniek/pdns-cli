use crate::rest_client::pdns_resource_client::{PowerDnsRestClient, ResponseEvent, RequestEvent, RequestPathProvider};
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::pdns::zone::Zone;
use crate::rest_client::errors::RestClientError;
use tokio::sync::oneshot::{Sender, Receiver};

pub struct ZoneResourceClient {
    pdns_resource_client: PowerDnsRestClient,
}

pub struct GetZoneRequestEvent {
    zone_name: String,
    response_channel: Sender<Box<GetZoneResponseEvent>>,
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

    pub async fn handle_request_event(&self, event_rx : Receiver<Box<GetZoneRequestEvent>>) {
        let client = &self.pdns_resource_client;

        client.handle_request_event(event_rx);
    }
}

impl GetZoneResponseEvent {
    fn new(result : Result<Zone, RestClientError>) -> GetZoneResponseEvent {
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

    fn response_channel(&self) -> Sender<Box<dyn ResponseEvent<Zone>>> {
        self.response_channel
    }

    fn request_path_provider(&self) -> Box<dyn RequestPathProvider> {
        Box::new(GetZoneRequestPathProvider::new(&self.zone_name))
    }
}

impl GetZoneRequestPathProvider {
    fn new(zone_name: &String) -> GetZoneRequestPathProvider {
        return GetZoneRequestPathProvider {
            request_path: format!("zones/{}", zone_name)
        }
    }
}

impl RequestPathProvider for GetZoneRequestPathProvider {
    fn provide_request_path(&self) -> String {
        self.request_path.clone()
    }
}