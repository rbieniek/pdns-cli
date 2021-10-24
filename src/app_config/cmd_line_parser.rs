use std::convert::TryFrom;

use clap::{App, Arg, ArgMatches};
use log::LevelFilter;
use uriparse::{Scheme, URI};

use crate::app_config::errors::{AppConfigError, UriPart};

const PARAM_BASE_URI: &'static str = "base-uri";
const PARAM_VERBOSITY: &'static str = "verbose";

pub struct ApplicationConfiguration {
    base_uri: String,
    log_level: LevelFilter,
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
    App::new("simple-cert-server")
        .version("1.0")
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