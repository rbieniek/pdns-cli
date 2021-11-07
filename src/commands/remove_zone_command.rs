use log::info;
use async_trait::async_trait;

use crate::commands::command_handler::CommandExecutor;
use crate::app_config::cmd_line_parser::CommandParameters;
use crate::rest_client::errors::RestClientError;

pub struct RemoveZoneCommand {
    base_uri: String,
    api_key: String,
    zone_name: String,
}

impl RemoveZoneCommand {
    pub fn new(base_uri: &String, api_key: &String, zone_name: &String) -> RemoveZoneCommand {
        RemoveZoneCommand {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
            zone_name: zone_name.clone(),
        }
    }
}

#[async_trait]
impl CommandExecutor for RemoveZoneCommand {
    async fn execute_command(&self, parameters: CommandParameters) -> Result<(), RestClientError> {
        if let CommandParameters::QueryZone { } = parameters {
            info!("Executing command query-zone, zone {}", &self.zone_name);

            Ok(())
        } else {
            Err(RestClientError::on_unspecified_error())
        }
    }
}