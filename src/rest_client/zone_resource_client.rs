// Copyright 2021 Cumulus Cloud Software und Consulting GmbH & Co KG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use log::info;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::pdns::zone::{Changetype, NewZone, Record, Rrset, Rrsets, RrsetType, Zone, ListZone};
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::rest_client::pdns_resource_client::{PnsServerResponse, PowerDnsRestClient};

pub struct ZoneResourceClient {
    pdns_resource_client: Arc<PowerDnsRestClient>,
    join_handles: Vec<JoinHandle<()>>,
}

pub struct QueryZoneRequestEvent {
    zone_name: String,
}

pub struct ListZonesRequestEvent {}

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

pub struct AddEntryRequestEvent {
    zone_name: String,
    record_key: String,
    record_type: String,
    record_values: Vec<String>,
    time_to_live: u32,
}

pub struct RemoveEntryRequestEvent {
    zone_name: String,
    record_key: String,
    record_type: String,
}

impl ZoneResourceClient {
    pub fn new(base_uri: &String, api_key: &String) -> ZoneResourceClient {
        ZoneResourceClient {
            pdns_resource_client: Arc::new(PowerDnsRestClient::new(ClientRequestBuilder::new(base_uri, api_key))),
            join_handles: Vec::new(),
        }
    }

    pub fn spawn_query_zone(&mut self,
                            request_rx: Receiver<QueryZoneRequestEvent>,
                            response_tx: Sender<PnsServerResponse<QueryZoneRequestEvent, Zone>>) {
        self.join_handles.push(tokio::spawn(handle_query_zone_request(self.pdns_resource_client.clone(),
                                                                      request_rx,
                                                                      response_tx)));
    }

    pub fn spawn_list_zones(&mut self,
                            request_rx: Receiver<ListZonesRequestEvent>,
                            response_tx: Sender<PnsServerResponse<ListZonesRequestEvent, Vec<ListZone>>>) {
        self.join_handles.push(tokio::spawn(handle_list_zones_request(self.pdns_resource_client.clone(),
                                                                     request_rx,
                                                                     response_tx)));
    }

    pub fn spawn_create_zone(&mut self,
                             request_rx: Receiver<CreateZoneRequestEvent>,
                             response_tx: Sender<PnsServerResponse<CreateZoneRequestEvent, Zone>>) {
        self.join_handles.push(tokio::spawn(handle_create_zone_request(self.pdns_resource_client.clone(),
                                                                       request_rx,
                                                                       response_tx)));
    }

    pub fn spawn_remove_zone(&mut self,
                             request_rx: Receiver<RemoveZoneRequestEvent>,
                             response_tx: Sender<PnsServerResponse<RemoveZoneRequestEvent, ()>>) {
        self.join_handles.push(tokio::spawn(handle_remove_zone_request(self.pdns_resource_client.clone(),
                                                                       request_rx,
                                                                       response_tx)));
    }

    pub fn spawn_add_entry(&mut self,
                           request_rx: Receiver<AddEntryRequestEvent>,
                           response_tx: Sender<PnsServerResponse<AddEntryRequestEvent, ()>>) {
        self.join_handles.push(tokio::spawn(handle_add_entry_request(self.pdns_resource_client.clone(),
                                                                     request_rx,
                                                                     response_tx)));
    }

    pub fn spawn_remove_entry(&mut self,
                              request_rx: Receiver<RemoveEntryRequestEvent>,
                              response_tx: Sender<PnsServerResponse<RemoveEntryRequestEvent, ()>>) {
        self.join_handles.push(tokio::spawn(handle_remove_entry_request(self.pdns_resource_client.clone(),
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

impl ListZonesRequestEvent {
    pub fn new() -> ListZonesRequestEvent {
        ListZonesRequestEvent {}
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

impl AddEntryRequestEvent {
    pub fn new(zone_name: &String, record_key: &String, record_type: &String,
               record_values: &Vec<String>, time_to_live: u32) -> AddEntryRequestEvent {
        AddEntryRequestEvent {
            zone_name: zone_name.clone(),
            record_key: record_key.clone(),
            record_type: record_type.clone(),
            record_values: record_values.clone(),
            time_to_live,
        }
    }
}

impl Display for AddEntryRequestEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "zone_name: {}, record_key: {}, record_type: {}, values: {}, ttl {}",
               self.zone_name.clone(), self.record_key.clone(), self.record_type.clone(),
               self.record_values.clone().join(","), self.time_to_live)
    }
}


impl RemoveEntryRequestEvent {
    pub fn new(zone_name: &String, record_key: &String, record_type: &String) -> RemoveEntryRequestEvent {
        RemoveEntryRequestEvent {
            zone_name: zone_name.clone(),
            record_key: record_key.clone(),
            record_type: record_type.clone(),
        }
    }
}

impl Display for RemoveEntryRequestEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "zone_name: {}, record_key: {}, record_type: {}",
               self.zone_name.clone(), self.record_key.clone(), self.record_type.clone())
    }
}

impl Drop for ZoneResourceClient {
    fn drop(&mut self) {
        for handle in self.join_handles.iter() {
            handle.abort();
        }
    }
}

async fn handle_query_zone_request(pdns_resource_client: Arc<PowerDnsRestClient>,
                                   request_rx: Receiver<QueryZoneRequestEvent>,
                                   response_tx: Sender<PnsServerResponse<QueryZoneRequestEvent, Zone>>) {
    pdns_resource_client
        .handle_get_request::<QueryZoneRequestEvent,
            Zone>(request_rx,
                  response_tx,
                  get_zone_request_path).await
}

async fn handle_list_zones_request(pdns_resource_client: Arc<PowerDnsRestClient>,
                                   request_rx: Receiver<ListZonesRequestEvent>,
                                   response_tx: Sender<PnsServerResponse<ListZonesRequestEvent, Vec<ListZone>>>) {
    pdns_resource_client
        .handle_get_request::<ListZonesRequestEvent,
            Vec<ListZone>>(request_rx,
                       response_tx,
                       list_zones_request_path).await
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

async fn handle_add_entry_request(pdns_resource_client: Arc<PowerDnsRestClient>,
                                  request_rx: Receiver<AddEntryRequestEvent>,
                                  response_tx: Sender<PnsServerResponse<AddEntryRequestEvent, ()>>) {
    pdns_resource_client
        .handle_patch_request::<AddEntryRequestEvent, Rrsets>(request_rx,
                                                              response_tx,
                                                              add_entry_request_path,
                                                              add_entry_body_provider).await
}

async fn handle_remove_entry_request(pdns_resource_client: Arc<PowerDnsRestClient>,
                                     request_rx: Receiver<RemoveEntryRequestEvent>,
                                     response_tx: Sender<PnsServerResponse<RemoveEntryRequestEvent, ()>>) {
    pdns_resource_client
        .handle_patch_request::<RemoveEntryRequestEvent, Rrsets>(request_rx,
                                                                 response_tx,
                                                                 remove_entry_request_path,
                                                                 remove_entry_body_provider).await
}

fn get_zone_request_path(request: &QueryZoneRequestEvent) -> String {
    format!("servers/localhost/zones/{}", &request.zone_name)
}

fn list_zones_request_path(_request: &ListZonesRequestEvent) -> String {
    format!("servers/localhost/zones")
}

fn create_zone_request_path(_request: &CreateZoneRequestEvent) -> String {
    "servers/localhost/zones".to_string()
}

fn remove_zone_request_path(request: &RemoveZoneRequestEvent) -> String {
    format!("servers/localhost/zones/{}", &request.zone_name)
}

fn add_entry_request_path(request: &AddEntryRequestEvent) -> String {
    format!("servers/localhost/zones/{}", &request.zone_name)
}

fn remove_entry_request_path(request: &RemoveEntryRequestEvent) -> String {
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

fn add_entry_body_provider(request: &AddEntryRequestEvent) -> Rrsets {
    let mut rrsets: Vec<Rrset> = Vec::new();
    let mut records: Vec<Record> = Vec::new();

    info!("create body for add-entry request: {}", request);

    for record_value in request.record_values.iter() {
        records.push(Record::new(&record_value, false));
    }

    rrsets.push(Rrset::new(&conditionally_qualify_key(&request.record_key,
                                                      &request.zone_name),
                           map_record_type(&request.record_type).unwrap(),
                           &Some(Changetype::Replace),
                           &Some(request.time_to_live),
                           &records,
                           &Vec::new()));

    Rrsets::new(&rrsets)
}

fn remove_entry_body_provider(request: &RemoveEntryRequestEvent) -> Rrsets {
    let mut rrsets: Vec<Rrset> = Vec::new();

    info!("create body for remove-entry request: {}", request);

    rrsets.push(Rrset::new(&conditionally_qualify_key(&request.record_key,
                                                      &request.zone_name),
                           map_record_type(&request.record_type).unwrap(),
                           &Some(Changetype::Delete),
                           &None,
                           &Vec::new(),
                           &Vec::new()));

    Rrsets::new(&rrsets)
}

fn canonicalize_name(name: &String) -> String {
    if !name.ends_with(".") {
        format!("{}.", name)
    } else {
        name.clone()
    }
}

fn map_record_type(record_type: &String) -> Option<RrsetType> {
    match record_type.as_str() {
        "A" => Some(RrsetType::A),
        "SOA" => Some(RrsetType::Soa),
        "NS" => Some(RrsetType::Ns),
        "PTR" => Some(RrsetType::Ptr),
        "TXT" => Some(RrsetType::Txt),
        "SRV" => Some(RrsetType::Srv),
        "CNAME" => Some(RrsetType::Cname),
        "AAAA" => Some(RrsetType::Aaaa),
        _ => None,
    }
}

fn conditionally_qualify_key(key: &String, zone_name: &String) -> String {
    if key.contains(zone_name) {
        canonicalize_name(key)
    } else {
        canonicalize_name(&format!("{}.{}", key, zone_name))
    }
}