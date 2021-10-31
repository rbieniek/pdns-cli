use log::info;
use async_trait::async_trait;

use crate::commands::command_handler::CommandExecutor;
use crate::app_config::cmd_line_parser::CommandParameters;
use crate::rest_client::errors::RestClientError;

pub struct RemoveZoneCommand {
    base_uri: String,
    api_key: String,
}

impl RemoveZoneCommand {
    pub fn new(base_uri: &String, api_key: &String) -> RemoveZoneCommand {
        RemoveZoneCommand {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
        }
    }
}

#[async_trait]
impl CommandExecutor for RemoveZoneCommand {
    async fn execute_command(&self, parameters: CommandParameters) -> Result<(), RestClientError> {
        if let CommandParameters::QueryZone { zone_name } = parameters {
            info!("Executing command query-zone, zone {}", &zone_name);

            Ok(())
        } else {
            Err(RestClientError::on_unspecified_error())
        }
    }
}