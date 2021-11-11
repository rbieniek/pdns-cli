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
use log::{info, LevelFilter};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::filter::threshold::ThresholdFilter;

use crate::app_config::cmd_line_parser::ApplicationConfiguration;
use crate::commands::command_handler::CommandHandler;

mod app_config;
mod rest_client;
mod pdns;
mod commands;

#[tokio::main]
async fn main() {
    match ApplicationConfiguration::process_command_line() {
        Ok(app_config) => {
            setup_logger(&app_config);

            info!("Using base URI {}", app_config.base_uri().clone());

            let command_handler = CommandHandler::new(&app_config.base_uri(),
                                                      &app_config.api_key(), &app_config.zone_name());
            let result = command_handler.execute_command(app_config.command());

            match result.await {
                Err(err) => panic!("Error executing ReST operation: {}", err),
                _ => {}
            }
        }
        Err(err) => panic!("Error parsing command line: {}", err)
    }
}

fn setup_logger(app_config: &ApplicationConfiguration) -> () {
    // Build a stdout logger.
    let stdout = ConsoleAppender::builder().target(Target::Stdout).build();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(app_config.log_level())))
                .build("stdout", Box::new(stdout)),
        )
        .build(
            Root::builder()
                .appender("stdout")
                .build(LevelFilter
                ::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    log4rs::init_config(config).unwrap();
}
