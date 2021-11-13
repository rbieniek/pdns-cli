use std::io::{stdout, Write};
use std::path::Path;

use async_trait::async_trait;
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
use reqwest::StatusCode;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot::channel;

use crate::app_config::cmd_line_parser::CommandParameters;
use crate::commands::command_handler::CommandExecutor;
use crate::pdns::server::{DaemonType, Server};
use crate::pdns::zone::{ListZone};
use crate::rest_client::errors::{RestClientError, RestClientErrorKind};
use crate::rest_client::pdns_resource_client::PnsServerResponse;
use crate::rest_client::server_resource_client::{QueryServerRequestEvent, ServerResourceClient};
use crate::rest_client::zone_resource_client::{ListZonesRequestEvent, ZoneResourceClient};

pub struct ListZonesCommand {
    base_uri: String,
    api_key: String,
    #[allow(dead_code)]
    zone_name: String,
}

impl ListZonesCommand {
    pub fn new(base_uri: &String, api_key: &String, zone_name: &String) -> ListZonesCommand {
        ListZonesCommand {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
            zone_name: zone_name.clone(),
        }
    }

    async fn execute_list_zones(&self) -> Result<String, RestClientError> {
        let mut zone_resource_client = ZoneResourceClient::new(&self.base_uri, &self.api_key);
        let (request_tx, request_rx) = channel::<ListZonesRequestEvent>();
        let (response_tx, response_rx) = channel::<PnsServerResponse<ListZonesRequestEvent, Vec<ListZone>>>();

        zone_resource_client.spawn_list_zones(request_rx, response_tx);

        match request_tx.send(ListZonesRequestEvent::new()) {
            Ok(()) => match response_rx.await {
                Ok(response_container) => match response_container.response() {
                    Ok(zones) => {
                        info!("Received zone data event for number of zones: {}", zones.len());

                        match serde_json::to_string_pretty(zones) {
                            Ok(json) => Ok(json.clone()),
                            Err(_) => Err(RestClientError::on_unspecified_error()),
                        }
                    }
                    Err(error) => match error.kind() {
                        RestClientErrorKind::PowerDnsServerError { status_code, server_error: _ } if status_code == StatusCode::NOT_FOUND => {
                            info!("Existing zone not found");

                            Err(error.clone())
                        }
                        _ => Err(error.clone())
                    }
                },
                Err(error) => Err(RestClientError::on_tokio_runtime_error(error.to_string())),
            }
            Err(_) => Err(RestClientError::on_unspecified_error()),
        }
    }
}

#[async_trait]
impl CommandExecutor for ListZonesCommand {
    async fn execute_command(&self, parameters: CommandParameters) -> Result<(), RestClientError> {
        if let CommandParameters::ListZone { output_file } = parameters {
            info!("Executing command list-zones");

            let mut server_resource_client = ServerResourceClient::new(&self.base_uri, &self.api_key);
            let (request_tx, request_rx) = channel::<QueryServerRequestEvent>();
            let (response_tx, response_rx) = channel::<PnsServerResponse<QueryServerRequestEvent, Server>>();

            server_resource_client.spawn_query(request_rx, response_tx);

            match request_tx.send(QueryServerRequestEvent::new()) {
                Ok(()) => match response_rx.await {
                    Ok(response_container) => match response_container.response() {
                        Ok(server) if server.daemon_type() == DaemonType::Authoritative => {
                            info!("Received Server data event: {}", server);

                            match self.execute_list_zones().await {
                                Ok(json) => match output_file {
                                    Some(path_name) => match File::create(Path::new(path_name.as_str())).await {
                                        Ok(mut file) => match file.write(json.as_bytes()).await {
                                            Ok(_) => Ok(()),
                                            Err(error) => Err(RestClientError::on_unspecified_error_message(&error.to_string())),
                                        },
                                        Err(error) => Err(RestClientError::on_unspecified_error_message(&error.to_string())),
                                    },
                                    None => match stdout().write(json.as_bytes()) {
                                        Ok(_) => Ok(()),
                                        Err(error) => Err(RestClientError::on_unspecified_error_message(&error.to_string())),
                                    },
                                },
                                Err(error) => Err(error),
                            }
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