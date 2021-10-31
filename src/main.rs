use log::{info, LevelFilter};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::filter::threshold::ThresholdFilter;
use tokio::sync::oneshot::channel;

use crate::app_config::cmd_line_parser::{ApplicationConfiguration, Command};
use crate::rest_client::errors::RestClientError;
use crate::rest_client::server_resource_client::{GetServerRequestEvent, GetServerResponseEvent, ServerResourceClient};
use crate::pdns::server::DaemonType;

mod app_config;
mod rest_client;
mod pdns;

#[tokio::main]
async fn main() {
    match ApplicationConfiguration::process_command_line() {
        Ok(app_config) => {
            setup_logger(&app_config);

            info!("Using base URI {}", app_config.base_uri().clone());

            let mut server_resource_client = ServerResourceClient::new();

            let result = match app_config.command() {
                Command::AddZone {
                    zone_name,
                    ignore_existing
                } => execute_add_zone(Command::AddZone {
                    zone_name,
                    ignore_existing,
                },
                                      app_config.base_uri().clone(),
                                      app_config.api_key().clone(),
                                      &mut server_resource_client).await,
                Command::RemoveZone { zone_name } => execute_remove_zone(Command::RemoveZone { zone_name }).await,
                Command::QueryZone { zone_name } => execute_query_zone(Command::QueryZone { zone_name }).await,
            };

            match result {
                Err(err) => panic!("Error executing ReST operation: {}", err),
                _ => {}
            }
        }
        Err(err) => panic!("Error parsing command line: {}", err)
    }
}

async fn execute_add_zone(command: Command,
                          base_uri: String,
                          api_key: String,
                          server_resource_client: &mut ServerResourceClient) -> Result<(), RestClientError> {
    if let Command::AddZone { zone_name, ignore_existing } = command {
        info!("Executing command add-zone, zone {}, ignore existing {}", &zone_name, ignore_existing);

        let (response_event_tx, response_event_rx) = channel::<GetServerResponseEvent>();
        let request_event_tx = server_resource_client.create_channel();

        match request_event_tx.send(GetServerRequestEvent::new(base_uri.clone(),
                                                               api_key.clone(),
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

async fn execute_remove_zone(command: Command) -> Result<(), RestClientError> {
    if let Command::RemoveZone { zone_name } = command {
        info!("Executing command remove-zone, zone {}", &zone_name);

        Ok(())
    } else {
        Err(RestClientError::on_unspecified_error())
    }
}

async fn execute_query_zone(command: Command) -> Result<(), RestClientError> {
    if let Command::QueryZone { zone_name } = command {
        info!("Executing command query-zone, zone {}", &zone_name);

        Ok(())
    } else {
        Err(RestClientError::on_unspecified_error())
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
