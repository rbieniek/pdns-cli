use log::{info, LevelFilter};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::filter::threshold::ThresholdFilter;

use crate::app_config::cmd_line_parser::{ApplicationConfiguration, Command};

mod app_config;
mod rest_client;
mod pdns;

#[tokio::main]
async fn main() {
    match ApplicationConfiguration::process_command_line() {
        Ok(app_config) => {
            setup_logger(&app_config);

            info!("Using base URI {}", app_config.base_uri().clone());

            match app_config.command() {
                Command::AddZone {
                    zone_name,
                    ignore_existing
                } => execute_add_zone(Command::AddZone {
                    zone_name,
                    ignore_existing,
                }),
                Command::RemoveZone { zone_name } => execute_remove_zone(Command::RemoveZone { zone_name }),
                Command::QueryZone { zone_name } => execute_query_zone(Command::QueryZone { zone_name }),
            }
        }
        Err(err) => panic!("Error parsing command line: {}", err)
    }
}

fn execute_add_zone(command: Command) {
    if let Command::AddZone { zone_name, ignore_existing} = command {
        info!("Executing command add-zone, zone {}, ignore existing {}", &zone_name, ignore_existing)
    }
}

fn execute_remove_zone(command: Command) {
    if let Command::RemoveZone { zone_name} = command {
        info!("Executing command remove-zone, zone {}", &zone_name)
    }
}

fn execute_query_zone(command: Command) {
    if let Command::QueryZone { zone_name} = command {
        info!("Executing command query-zone, zone {}", &zone_name)


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
