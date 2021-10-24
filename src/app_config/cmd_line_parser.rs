use log::LevelFilter;
use clap::{ArgMatches, Arg, App};
use uriparse::URI;
use std::convert::TryFrom;
use crate::app_config::errors::AppConfigError;

const PARAM_BASE_URI: &'static str = "base-uri";
const PARAM_VERBOSITY: &'static str = "verbose";

pub struct ApplicationConfiguration {
    base_uri: String,
    log_level: LevelFilter,
}

fn parse_command_line() -> ArgMatches {
    App::new("simple-cert-server")
        .version("1.0")
        .author("Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>")
        .arg(Arg::new(PARAM_BASE_URI)
            .about("PowerDNS ReST API base URI")
            .long(PARAM_BASE_URI)
            .short('u')
            .takes_value(true)
            .required(false)
            .validator(|value| {
                match URI::try_from(value) {
                    Ok(_) =>  {

                        Ok(())
                    },
                    Err(parser_error) => Err(AppConfigError::on_malformed_base_uri(&value.to_string(), &parser_error))
                }
            })
        )
        .arg(
            Arg::new(PARAM_VERBOSITY)
                .about("Change verbosity of output")
                .long("verbose")
                .short('v')
                .multiple_occurrences(true)
        )
        .get_matches()
}