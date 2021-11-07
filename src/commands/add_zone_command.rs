use async_trait::async_trait;
use log::info;
use reqwest::StatusCode;
use tokio::sync::oneshot::channel;

use crate::app_config::cmd_line_parser::CommandParameters;
use crate::commands::command_handler::CommandExecutor;
use crate::pdns::server::DaemonType;
use crate::pdns::zone::Zone;
use crate::rest_client::errors::{RestClientError, RestClientErrorKind};
use crate::rest_client::pdns_resource_client::PnsServerResponse;
use crate::rest_client::server_resource_client::{GetServerRequestEvent, GetServerResponseEvent, ServerResourceClient};
use crate::rest_client::zone_resource_client::{QueryZoneRequestEvent, ZoneResourceClient};

pub struct AddZoneCommand {
    base_uri: String,
    api_key: String,
    zone_name: String,
}

impl AddZoneCommand {
    pub fn new(base_uri: &String, api_key: &String, zone_name: &String) -> AddZoneCommand {
        AddZoneCommand {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
            zone_name: zone_name.clone(),
        }
    }

    async fn execute_get_zone(&self, refresh: u32, retry: u32, expire: u32,
                              neg_caching: u32, masters: &Vec<String>,
                              nameservers: &Vec<String>, account: &String) -> Result<(), RestClientError> {
        let mut zone_resource_client = ZoneResourceClient::new(&self.base_uri, &self.api_key);
        let (request_tx, request_rx) = channel::<QueryZoneRequestEvent>();
        let (response_tx, response_rx) = channel::<PnsServerResponse<QueryZoneRequestEvent, Zone>>();

        zone_resource_client.spawn_get(request_rx, response_tx);

        match request_tx.send(QueryZoneRequestEvent::new(&self.zone_name)) {
            Ok(()) => match response_rx.await {
                Ok(response_container) => match response_container.response() {
                    Ok(zone) => {
                        info!("Received Zone data event: {}", zone);

                        Ok(())
                    }
                    Err(error) => match error.kind() {
                        RestClientErrorKind::PowerDnsServerError { status_code, server_error } if status_code == StatusCode::NOT_FOUND => {
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
}

#[async_trait]
impl CommandExecutor for AddZoneCommand {
    async fn execute_command(&self,
                             command: CommandParameters) -> Result<(), RestClientError> {
        if let CommandParameters::AddZone {
            refresh, retry, expire,
            neg_caching, masters, nameservers, account
        } = command {
            info!("Executing command add-zone, zone {}", &self.zone_name);

            let mut server_resource_client = ServerResourceClient::new();
            let (response_event_tx, response_event_rx) = channel::<GetServerResponseEvent>();
            let request_event_tx = server_resource_client.create_channel();

            match request_event_tx.send(GetServerRequestEvent::new(self.base_uri.clone(),
                                                                   self.api_key.clone(),
                                                                   response_event_tx)) {
                Ok(()) => match response_event_rx.await {
                    Ok(response) => match response.result() {
                        Ok(server_response) if server_response.daemon_type() == DaemonType::Authoritative => {
                            info!("Received Server data event: {}", server_response);

                            self.execute_get_zone(refresh, retry, expire,
                                                  neg_caching, &masters, &nameservers,
                                                  &account).await
                        }
                        Ok(_) => Err(RestClientError::on_unspecified_error()),
                        Err(error) => Err(error),
                    },
                    Err(error) => Err(RestClientError::on_tokio_runtime_error(error.to_string())),
                },
                Err(_) => Err(RestClientError::on_unspecified_error()),
            }
        } else {
            Err(RestClientError::on_unspecified_error())
        }
    }
}