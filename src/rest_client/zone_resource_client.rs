use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::pdns::zone::{NewZone, Record, Rrset, RrsetType, Zone};
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
    refresh: u32,
    retry: u32,
    expire: u32,
    neg_caching: u32,
    masters: Vec<String>,
    nameservers: Vec<String>,
    account: String,
}

pub struct RemoveZoneRequestEvent {
    zone_name: String,
}

impl ZoneResourceClient {
    pub fn new(base_uri: &String, api_key: &String) -> ZoneResourceClient {
        ZoneResourceClient {
            pdns_resource_client: Arc::new(PowerDnsRestClient::new(ClientRequestBuilder::new(base_uri, api_key))),
            join_handles: Vec::new(),
        }
    }

    pub fn spawn_query(&mut self,
                       request_rx: Receiver<QueryZoneRequestEvent>,
                       response_tx: Sender<PnsServerResponse<QueryZoneRequestEvent, Zone>>) {
        self.join_handles.push(tokio::spawn(handle_get_request(self.pdns_resource_client.clone(),
                                                               request_rx,
                                                               response_tx)));
    }

    pub fn spawn_create(&mut self,
                        request_rx: Receiver<CreateZoneRequestEvent>,
                        response_tx: Sender<PnsServerResponse<CreateZoneRequestEvent, Zone>>) {
        self.join_handles.push(tokio::spawn(handle_create_zone_request(self.pdns_resource_client.clone(),
                                                                       request_rx,
                                                                       response_tx)));
    }

    pub fn spawn_remove(&mut self,
                        request_rx: Receiver<RemoveZoneRequestEvent>,
                        response_tx: Sender<PnsServerResponse<RemoveZoneRequestEvent, ()>>) {
        self.join_handles.push(tokio::spawn(handle_remove_zone_request(self.pdns_resource_client.clone(),
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

impl CreateZoneRequestEvent {
    pub fn new(zone_name: &String, refresh: u32, retry: u32, expire: u32, neg_caching: u32,
               masters: &Vec<String>, nameservers: &Vec<String>, account: &String) -> CreateZoneRequestEvent {
        CreateZoneRequestEvent {
            zone_name: zone_name.clone(),
            refresh,
            retry,
            expire,
            neg_caching,
            masters: masters.clone(),
            nameservers: nameservers.clone(),
            account: account.clone(),
        }
    }
}

impl RemoveZoneRequestEvent {
    pub fn new(zone_name: &String) -> RemoveZoneRequestEvent {
        RemoveZoneRequestEvent {
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
                           create_zone_request_path,
                           create_zone_body_provider).await
}


async fn handle_remove_zone_request(pdns_resource_client: Arc<PowerDnsRestClient>,
                                    request_rx: Receiver<RemoveZoneRequestEvent>,
                                    response_tx: Sender<PnsServerResponse<RemoveZoneRequestEvent, ()>>) {
    pdns_resource_client
        .handle_delete_request::<RemoveZoneRequestEvent>(request_rx,
                                                         response_tx,
                                                         remove_zone_request_path).await
}

fn get_zone_request_path(request: &QueryZoneRequestEvent) -> String {
    format!("servers/localhost/zones/{}", &request.zone_name)
}

fn create_zone_request_path(_request: &CreateZoneRequestEvent) -> String {
    "servers/localhost/zones".to_string()
}

fn remove_zone_request_path(request: &RemoveZoneRequestEvent) -> String {
    format!("servers/localhost/zones/{}", &request.zone_name)
}


fn create_zone_body_provider(request: &CreateZoneRequestEvent) -> NewZone {
    let mut rrsets: Vec<Rrset> = Vec::new();
    let utc: DateTime<Utc> = Utc::now();
    let serial = utc.format("%Y%m%d").to_string();
    let mut nameservers: Vec<String> = Vec::new();
    let mut masters: Vec<String> = Vec::new();

    for value in request.nameservers.iter() {
        nameservers.push(canonicalize_name(value));
    }
    for value in request.masters.iter() {
        masters.push(canonicalize_name(value));
    }

    rrsets.push(Rrset::new(&canonicalize_name(&request.zone_name), RrsetType::Soa,
                           &None,
                           &Some(request.refresh),
                           &vec![
                               Record::new(&format!("{} {}.{} {}01 {} {} {} {}",
                                                    canonicalize_name(&request.zone_name),
                                                    &request.account,
                                                    canonicalize_name(&request.zone_name),
                                                    serial,
                                                    request.refresh,
                                                    request.retry,
                                                    request.expire,
                                                    request.neg_caching),
                                           false)
                           ],
                           &Vec::new()));

    NewZone::new(&request.zone_name, &rrsets, &masters, &nameservers,
                 false, None, false, false,
                 None, None)
}

fn canonicalize_name(name: &String) -> String {
    if !name.ends_with(".") {
        format!("{}.", name)
    } else {
        name.clone()
    }
}