use log::info;
use tokio::sync::oneshot::channel;
use async_trait::async_trait;

use crate::app_config::cmd_line_parser::CommandParameters;
use crate::rest_client::errors::RestClientError;
use crate::rest_client::server_resource_client::{GetServerRequestEvent, GetServerResponseEvent, ServerResourceClient};
use crate::pdns::server::DaemonType;
use crate::commands::command_handler::CommandExecutor;

pub struct AddZoneCommand {
    base_uri: String,
    api_key: String,
}

impl AddZoneCommand {
    pub fn new(base_uri: &String, api_key: &String) -> AddZoneCommand {
        AddZoneCommand {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
        }
    }
}

#[async_trait]
impl CommandExecutor for AddZoneCommand {
    async fn execute_command(&self,
                              command: CommandParameters) -> Result<(), RestClientError> {
        if let CommandParameters::AddZone { zone_name, ignore_existing } = command {
            info!("Executing command add-zone, zone {}, ignore existing {}", &zone_name, ignore_existing);

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

                            Ok(())
                        },
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