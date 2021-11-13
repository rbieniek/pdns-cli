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
use std::collections::HashMap;

use async_trait::async_trait;

use crate::app_config::cmd_line_parser::{Command, CommandKind, CommandParameters};
use crate::commands::add_zone_command::AddZoneCommand;
use crate::rest_client::errors::RestClientError;
use crate::commands::query_zone_command::QueryZoneCommand;
use crate::commands::remove_zone_command::RemoveZoneCommand;
use crate::commands::add_entry_command::AddEntryCommand;
use crate::commands::remove_entry_command::RemoveEntryCommand;

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
        executors.insert(CommandKind::AddEntry, Box::new(AddEntryCommand::new(&base_uri, &api_key, zone_name)));
        executors.insert(CommandKind::RemoveEntry, Box::new(RemoveEntryCommand::new(&base_uri, &api_key, zone_name)));

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
            None => Err(RestClientError::on_unspecified_error_message(&format!("Unknown operation: {}", command.kind()))),
        }
    }
}