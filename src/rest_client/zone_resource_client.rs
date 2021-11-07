use std::sync::Arc;

use tokio::sync::oneshot::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::pdns::struct_type::StructType;
use crate::pdns::zone::{NewZone, Zone, Rrset};
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::rest_client::pdns_resource_client::{PnsServerResponse, PowerDnsRestClient};

pub struct ZoneResourceClient {
    pdns_resource_client: Arc<PowerDnsRestClient>,
    join_handles: Vec<JoinHandle<()>>,
}

pub struct QueryZoneRequestEvent {
    zone_name: String,
}

pub struct CreateZoneRequestEvent {
    zone_name: String,
}

impl ZoneResourceClient {
    pub fn new(base_uri: &String, api_key: &String) -> ZoneResourceClient {
        ZoneResourceClient {
            pdns_resource_client: Arc::new(PowerDnsRestClient::new(ClientRequestBuilder::new(base_uri, api_key))),
            join_handles: Vec::new(),
        }
    }

    pub fn spawn_get(&mut self,
                     request_rx: Receiver<QueryZoneRequestEvent>,
                     response_tx: Sender<PnsServerResponse<QueryZoneRequestEvent, Zone>>) {
        self.join_handles.push(tokio::spawn(handle_get_request(self.pdns_resource_client.clone(),
                                                               request_rx,
                                                               response_tx)));
    }
}

impl QueryZoneRequestEvent {
    pub fn new(zone_name: &String) -> QueryZoneRequestEvent {
        QueryZoneRequestEvent {
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

async fn handle_get_request(pdns_resource_client: Arc<PowerDnsRestClient>,
                            request_rx: Receiver<QueryZoneRequestEvent>,
                            response_tx: Sender<PnsServerResponse<QueryZoneRequestEvent, Zone>>) {
    pdns_resource_client
        .handle_get_request::<QueryZoneRequestEvent,
            Zone>(request_rx,
                  response_tx,
                  get_zone_request_path).await
}

async fn handle_create_zone_request(pdns_resource_client: Arc<PowerDnsRestClient>,
                                    request_rx: Receiver<CreateZoneRequestEvent>,
                                    response_tx: Sender<PnsServerResponse<CreateZoneRequestEvent, Zone>>) {
    pdns_resource_client
        .handle_post_request::<CreateZoneRequestEvent,
            Zone, NewZone>(request_rx,
                           response_tx,
                           post_zone_request,
                           create_zone_body_provider).await
}

fn get_zone_request_path(request: &QueryZoneRequestEvent) -> String {
    format!("servers/localhost/zones/{}", &request.zone_name)
}

fn post_zone_request(_request: &CreateZoneRequestEvent) -> String {
    "servers/localhost/zones".to_string()
}

fn create_zone_body_provider(request: &CreateZoneRequestEvent) -> NewZone {
    let mut rrsets: Vec<Rrset> = Vec::new();
    let mut nameservers: Vec<String> = Vec::new();

    NewZone::new(&request.zone_name, &rrsets, &vec![], &nameservers,
                 false, None, false, false,
                 None, None)
}