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
use crate::commands::command_handler::CommandExecutor;
use crate::app_config::cmd_line_parser::CommandParameters;
use crate::rest_client::errors::RestClientError;

pub struct QueryZoneCommand {
    base_uri: String,
    api_key: String,
    zone_name: String,
}

impl QueryZoneCommand {
    pub fn new(base_uri: &String, api_key: &String, zone_name: &String) -> QueryZoneCommand {
        QueryZoneCommand {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
            zone_name: zone_name.clone(),
        }
    }
}

#[async_trait]
impl CommandExecutor for QueryZoneCommand {
    async fn execute_command(&self, parameters: CommandParameters) -> Result<(), RestClientError> {
        if let CommandParameters::RemoveZone { } = parameters {
            info!("Executing command remove-zone, zone {}", &self.zone_name);

            Ok(())
        } else {
            Err(RestClientError::on_unspecified_error())
        }
    }
}