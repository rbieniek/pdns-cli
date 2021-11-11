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

use log::info;
use async_trait::async_trait;
use reqwest::StatusCode;
use tokio::sync::oneshot::channel;

use crate::app_config::cmd_line_parser::CommandParameters;
use crate::commands::command_handler::CommandExecutor;
use crate::pdns::server::{DaemonType, Server};
use crate::pdns::zone::Zone;
use crate::rest_client::errors::{RestClientError, RestClientErrorKind};
use crate::rest_client::pdns_resource_client::PnsServerResponse;
use crate::rest_client::server_resource_client::{QueryServerRequestEvent, ServerResourceClient};
use crate::rest_client::zone_resource_client::{QueryZoneRequestEvent, RemoveZoneRequestEvent, ZoneResourceClient, AddEntryRequestEvent};

pub struct AddEntryCommand {
    base_uri: String,
    api_key: String,
    zone_name: String,
}

impl AddEntryCommand {
    pub fn new(base_uri: &String, api_key: &String, zone_name: &String) -> AddEntryCommand {
        AddEntryCommand {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
            zone_name: zone_name.clone(),
        }
    }

    async fn execute_get_zone(&self, record_key: &String, record_type: &String, record_value: &Vec<String>, time_to_live: u32) -> Result<(), RestClientError> {
        let mut zone_resource_client = ZoneResourceClient::new(&self.base_uri, &self.api_key);
        let (request_tx, request_rx) = channel::<QueryZoneRequestEvent>();
        let (response_tx, response_rx) = channel::<PnsServerResponse<QueryZoneRequestEvent, Zone>>();

        zone_resource_client.spawn_query_zone(request_rx, response_tx);

        match request_tx.send(QueryZoneRequestEvent::new(&self.zone_name)) {
            Ok(()) => match response_rx.await {
                Ok(response_container) => match response_container.response() {
                    Ok(zone) => {
                        info!("Received zone data event: {}", zone);

                        let mut found = false;

                        for rrset in zone.rrsets().into_iter() {
                            if rrset.name().eq(&record_key) {
                                found = true;
                                break;
                            }
                        }

                        if(found) {
                            Err(RestClientError::on_unspecified_error_message(fmt!("Record already exists: {}", &record_key)))
                        } else {
                            self.execute_add_entry(record_key, record_type, record_value, time_to_live).await
                        }
                    }
                    Err(error) => match error.kind() {
                        RestClientErrorKind::PowerDnsServerError { status_code, server_error: _ } if status_code == StatusCode::NOT_FOUND => {
                            info!("Existing zone not found");

                            Ok(())
                        }
                        _ => Err(error.clone())
                    }
                },
                Err(error) => Err(RestClientError::on_tokio_runtime_error(error.to_string())),
            }
            Err(_) => Err(RestClientError::on_unspecified_error()),
        }
    }

    async fn execute_add_entry(&self, record_key: &String, record_type: &String, record_value: &Vec<String>, time_to_live: u32) -> Result<(), RestClientError> {
        let mut zone_resource_client = ZoneResourceClient::new(&self.base_uri, &self.api_key);
        let (request_tx, request_rx) = channel::<AddEntryRequestEvent>();
        let (response_tx, response_rx) = channel::<PnsServerResponse<AddEntryRequestEvent, ()>>();

        zone_resource_client.spawn_add_entry(request_rx, response_tx);

        match request_tx.send(AddEntryRequestEvent::new(&self.zone_name)) {
            Ok(()) => match response_rx.await {
                Ok(response_container) => match response_container.response() {
                    Ok(()) => {
                        info!("Received remove zone data event");

                        Ok(())
                    }
                    Err(error) => Err(error.clone()),
                },
                Err(error) => Err(RestClientError::on_tokio_runtime_error(error.to_string())),
            }
            Err(_) => Err(RestClientError::on_unspecified_error()),
        }
    }
}

#[async_trait]
impl CommandExecutor for AddEntryCommand {
    async fn execute_command(&self, parameters: CommandParameters) -> Result<(), RestClientError> {
        if let CommandParameters::AddEntry {
            record_key, record_type, record_value, time_to_live,
        } = parameters {
            info!("Executing command remove-zone, zone {}", &self.zone_name);

            let mut server_resource_client = ServerResourceClient::new(&self.base_uri, &self.api_key);
            let (request_tx, request_rx) = channel::<QueryServerRequestEvent>();
            let (response_tx, response_rx) = channel::<PnsServerResponse<QueryServerRequestEvent, Server>>();

            server_resource_client.spawn_query(request_rx, response_tx);

            match request_tx.send(QueryServerRequestEvent::new()) {
                Ok(()) => match response_rx.await {
                    Ok(response_container) => match response_container.response() {
                        Ok(server) if server.daemon_type() == DaemonType::Authoritative => {
                            info!("Received Server data event: {}", server);

                            self.execute_get_zone(&record_key, &record_type, &record_value, time_to_live).await
                        }
                        Ok(_) => Err(RestClientError::on_unspecified_error()),
                        Err(error) => Err(error.clone()),
                    },
                    Err(error) => Err(RestClientError::on_tokio_runtime_error(error.to_string())),
                }
                Err(_) => Err(RestClientError::on_unspecified_error()),
            }
        } else {
            Err(RestClientError::on_unspecified_error())
        }
    }
}
