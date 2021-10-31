use log::info;
use async_trait::async_trait;
use crate::commands::command_handler::CommandExecutor;
use crate::app_config::cmd_line_parser::CommandParameters;
use crate::rest_client::errors::RestClientError;

pub struct QueryZoneCommand {
    base_uri: String,
    api_key: String,
}

impl QueryZoneCommand {
    pub fn new(base_uri: &String, api_key: &String) -> QueryZoneCommand {
        QueryZoneCommand {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
        }
    }
}

#[async_trait]
impl CommandExecutor for QueryZoneCommand {
    async fn execute_command(&self, parameters: CommandParameters) -> Result<(), RestClientError> {
        if let CommandParameters::RemoveZone { zone_name } = parameters {
            info!("Executing command remove-zone, zone {}", &zone_name);

            Ok(())
        } else {
            Err(RestClientError::on_unspecified_error())
        }
    }
}