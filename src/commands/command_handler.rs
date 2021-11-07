use std::collections::HashMap;

use async_trait::async_trait;

use crate::app_config::cmd_line_parser::{Command, CommandKind, CommandParameters};
use crate::commands::add_zone_command::AddZoneCommand;
use crate::rest_client::errors::RestClientError;
use crate::commands::query_zone_command::QueryZoneCommand;
use crate::commands::remove_zone_command::RemoveZoneCommand;

pub struct CommandHandler {
    executors: HashMap<CommandKind, Box<dyn CommandExecutor>>,
}

#[async_trait]
pub(crate) trait CommandExecutor {
    async fn execute_command(&self, parameters: CommandParameters) -> Result<(), RestClientError>;
}

impl CommandHandler {
    pub fn new(base_uri: &String, api_key: &String, zone_name: &String) -> CommandHandler {
        let mut executors: HashMap<CommandKind, Box<dyn CommandExecutor>> = HashMap::new();

        executors.insert(CommandKind::AddZone, Box::new(AddZoneCommand::new(&base_uri, &api_key, zone_name)));
        executors.insert(CommandKind::QueryZone, Box::new(QueryZoneCommand::new(&base_uri, &api_key, zone_name)));
        executors.insert(CommandKind::RemoveZone, Box::new(RemoveZoneCommand::new(&base_uri, &api_key, zone_name)));

        CommandHandler {
            executors,
        }
    }

    pub async fn execute_command(&self, command: Command) -> Result<(), RestClientError> {
        match self.executors.get(&command.kind()) {
            Some(command_executor) => match command_executor
                .execute_command(command.parameters())
                .await {
                Ok(()) => Ok(()),
                Err(error) => Err(error),
            },
            None => Err(RestClientError::on_unspecified_error()),
        }
    }
}