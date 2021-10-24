use std::convert::TryFrom;

use clap::{App, Arg, ArgMatches};
use fancy_regex::Regex;
use log::LevelFilter;
use uriparse::{Scheme, URI};

use crate::app_config::errors::{AppConfigError, UriPart};

const PARAM_BASE_URI: &'static str = "base-uri";
const PARAM_ZONE_NAME: &'static str = "zone-name";
const PARAM_IGNORE_EXISTING: &'static str = "ignore-existing";
const PARAM_VERBOSITY: &'static str = "verbose";

pub struct ApplicationConfiguration {
    base_uri: String,
    log_level: LevelFilter,
}

pub enum Command {
    AddZone {
        zone_name: String,
        ignore_existing: bool,
    },
    RemoveZone {
        zone_name: String,
    },
}

impl ApplicationConfiguration {
    /// Parse the command line to build the application configuration structure
    pub fn process_command_line() -> Result<ApplicationConfiguration, AppConfigError> {
        let matches = parse_command_line();

        let level = match matches.occurrences_of(PARAM_VERBOSITY) {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };

        Ok(ApplicationConfiguration {
            base_uri: matches.value_of(PARAM_BASE_URI).unwrap().to_string(),
            log_level: level,
        })
    }

    /// Return the bind address specified on the command line (if any).
    /// If the bind address was omitted, provide the fallback value "127.0.0.1:8080"
    pub fn base_uri(&self) -> String {
        self.base_uri.clone()
    }

    pub fn log_level(&self) -> LevelFilter {
        self.log_level
    }
}

pub fn parse_command_line() -> ArgMatches {
    App::new("pdns-cli")
        .version("1.0")
        .about("Modify PowerDNS instance data")
        .author("Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>")
        .arg(Arg::new(PARAM_BASE_URI)
            .about("PowerDNS ReST API base URI")
            .long(PARAM_BASE_URI)
            .short('u')
            .takes_value(true)
            .required(true)
            .validator(|value| {
                match URI::try_from(value) {
                    Ok(base_uri) => verify_base_uri(&base_uri),
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
        .subcommand(App::new("add-zone")
            .about("Add zone to PowerDNS instance")
            .arg(Arg::new(PARAM_ZONE_NAME)
                .about("Zone name")
                .long(PARAM_ZONE_NAME)
                .short('n')
                .takes_value(true)
                .required(true)
            )
        )
        .get_matches()
}

fn verify_base_uri(base_uri: &URI) -> Result<(), AppConfigError> {
    match base_uri.scheme() {
        Scheme::HTTP | Scheme::HTTPS => match base_uri.username() {
            Some(user_name) => Err(AppConfigError::on_invalid_uri_part(&base_uri.to_string(),
                                                                       &UriPart::Username {
                                                                           user: user_name.to_string()
                                                                       })),
            None => match base_uri.password() {
                Some(password) => Err(AppConfigError::on_invalid_uri_part(&base_uri.to_string(),
                                                                          &UriPart::Password {
                                                                              password: password.to_string()
                                                                          })),
                None => if base_uri.path().to_string() == "/" {
                    match base_uri.query() {
                        Some(query) => Err(AppConfigError::on_invalid_uri_part(&base_uri.to_string(),
                                                                               &UriPart::Query {
                                                                                   query: query.to_string()
                                                                               })),
                        None => match base_uri.fragment() {
                            Some(fragment) => Err(AppConfigError::on_invalid_uri_part(&base_uri.to_string(),
                                                                                      &UriPart::Fragment {
                                                                                          fragment: fragment.to_string()
                                                                                      })),
                            None => Ok(())
                        }
                    }
                } else {
                    Err(AppConfigError::on_invalid_uri_part(&base_uri.to_string(),
                                                            &UriPart::Path {
                                                                path: base_uri.path().to_string()
                                                            }))
                }
            }
        },
        _ => Err(AppConfigError::on_invalid_uri_part(&base_uri.to_string(),
                                                     &UriPart::Scheme {
                                                         scheme: base_uri.scheme().to_string()
                                                     }))
    }
}

fn verify_zone_name(value: &str) -> Result<(), AppConfigError> {
    let re = Regex::new("^((?!-)[A-Za-z0-9-]{1,63}(?<!-)\\.)+[A-Za-z]{2,6}$").unwrap();

    match re.is_match(value) {
        Ok(result) => match result {
            true => Ok(()),
            false => Err(AppConfigError::on_malformed_zone_name(&value.to_string(), &"".to_string())),
        },
        Err(error) => Err(AppConfigError::on_malformed_zone_name(&value.to_string(), &error.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use crate::app_config::cmd_line_parser::verify_zone_name;

    #[test]
    fn should_validate_valid_zone_name() {
        assert_eq!(verify_zone_name("ccsac.de"), Ok(()))
    }

    #[test]
    fn should_validate_valid_rev_zone_name() {
        assert_eq!(verify_zone_name("26.16.172.in-addr.arpa"), Ok(()))
    }
}