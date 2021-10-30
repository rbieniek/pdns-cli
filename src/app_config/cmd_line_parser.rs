use std::convert::TryFrom;

use clap::{App, Arg, ArgMatches};
use fancy_regex::Regex;
use log::LevelFilter;
use uriparse::{Scheme, URI};

use crate::app_config::errors::{AppConfigError, UriPart};

const PARAM_BASE_URI: &'static str = "base-uri";
const PARAM_API_KEY: &'static str = "api-key";
const PARAM_ZONE_NAME: &'static str = "zone-name";
const PARAM_IGNORE_EXISTING: &'static str = "ignore-existing";
const PARAM_VERBOSITY: &'static str = "verbose";
const SUBCOMMAND_ADD_ZONE: &'static str = "add-zone";
const SUBCOMMAND_REMOVE_ZONE: &'static str = "remove-zone";
const SUBCOMMAND_QUERY_ZONE: &'static str = "query-zone";


pub struct ApplicationConfiguration {
    base_uri: String,
    api_key: String,
    log_level: LevelFilter,
    command: Command,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    AddZone {
        zone_name: String,
        ignore_existing: bool,
    },
    RemoveZone {
        zone_name: String,
    },
    QueryZone {
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

        let command_add_zone = if let Some(command) = matches.subcommand_matches(SUBCOMMAND_ADD_ZONE) {
            Some(Command::AddZone {
                zone_name: command.value_of(PARAM_ZONE_NAME).unwrap().to_string(),
                ignore_existing: command.is_present(PARAM_IGNORE_EXISTING),
            })
        } else { None };
        let command_query_zone = if let Some(command) = matches.subcommand_matches(SUBCOMMAND_QUERY_ZONE) {
            Some(Command::QueryZone {
                zone_name: command.value_of(PARAM_ZONE_NAME).unwrap().to_string(),
            })
        } else { None };
        let command_remove_zone = if let Some(command) = matches.subcommand_matches(SUBCOMMAND_REMOVE_ZONE) {
            Some(Command::RemoveZone {
                zone_name: command.value_of(PARAM_ZONE_NAME).unwrap().to_string(),
            })
        } else { None };

        match command_add_zone.or(command_query_zone).or(command_remove_zone) {
            Some(command) => Ok(ApplicationConfiguration {
                base_uri: matches.value_of(PARAM_BASE_URI).unwrap().to_string(),
                api_key:  matches.value_of(PARAM_API_KEY).unwrap().to_string(),
                log_level: level,
                command
            }),
            None => Err(AppConfigError::on_missing_command())
        }
    }

    /// Return the bind address specified on the command line (if any).
    /// If the bind address was omitted, provide the fallback value "127.0.0.1:8080"
    pub fn base_uri(&self) -> String {
        self.base_uri.clone()
    }

    pub fn api_key(&self) -> String {
        self.api_key.clone()
    }

    pub fn command(&self) -> Command {
        self.command.clone()
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
        .arg(Arg::new(PARAM_API_KEY)
            .about("PowerDNS ReST API key")
            .long(PARAM_BASE_URI)
            .short('k')
            .takes_value(true)
            .required(true)
        )
        .arg(
            Arg::new(PARAM_VERBOSITY)
                .about("Change verbosity of output")
                .long(PARAM_VERBOSITY)
                .short('v')
                .multiple_occurrences(true)
        )
        .subcommand(App::new(SUBCOMMAND_ADD_ZONE)
            .about("Add zone to PowerDNS instance")
            .arg(Arg::new(PARAM_ZONE_NAME)
                .about("Zone name")
                .long(PARAM_ZONE_NAME)
                .short('n')
                .takes_value(true)
                .required(true)
                .validator(|value| verify_zone_name(value)))
            .arg(Arg::new(PARAM_IGNORE_EXISTING)
                .about("Ignore existing zone")
                .long(PARAM_IGNORE_EXISTING)
                .short('e')
                .required(false)
                .takes_value(false)))
        .subcommand(App::new(SUBCOMMAND_QUERY_ZONE)
            .about("Add zone to PowerDNS instance")
            .arg(Arg::new(PARAM_ZONE_NAME)
                .about("Zone name")
                .long(PARAM_ZONE_NAME)
                .short('n')
                .takes_value(true)
                .required(true)
                .validator(|value| verify_zone_name(value))))
        .subcommand(App::new(SUBCOMMAND_REMOVE_ZONE)
            .about("Add zone to PowerDNS instance")
            .arg(Arg::new(PARAM_ZONE_NAME)
                .about("Zone name")
                .long(PARAM_ZONE_NAME)
                .short('n')
                .takes_value(true)
                .required(true)
                .validator(|value| verify_zone_name(value))))
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
    use crate::app_config::cmd_line_parser::{verify_zone_name, verify_base_uri};
    use uriparse::URI;
    use std::convert::TryFrom;
    use crate::app_config::errors::{AppConfigErrorKind, UriPart};

    #[test]
    fn should_verify_valid_base_uri_with_http() {
        let uri = URI::try_from("http://localhost:8080/").unwrap();
        let result = verify_base_uri(&uri);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn should_verify_valid_base_uri_with_https() {
        let uri = URI::try_from("https://localhost:8080/").unwrap();
        let result = verify_base_uri(&uri);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn should_fail_base_uri_with_bad_scheme() {
        let uri_str = "ftp://localhost:8080/";
        let uri = URI::try_from(uri_str).unwrap();
        let result = verify_base_uri(&uri);

        assert_eq!(result.is_err(), true);

        let app_error = result.unwrap_err();

        assert_eq!(app_error.kind, AppConfigErrorKind::InvalidUriPart {
            base_uri: uri_str.to_string(),
            uri_part: UriPart::Scheme {
                scheme: "ftp".to_string()
            }
        })
    }

    #[test]
    fn should_fail_base_uri_with_username() {
        let uri_str = "http://foo@localhost:8080/";
        let uri = URI::try_from(uri_str).unwrap();
        let result = verify_base_uri(&uri);

        assert_eq!(result.is_err(), true);

        let app_error = result.unwrap_err();

        assert_eq!(app_error.kind, AppConfigErrorKind::InvalidUriPart {
            base_uri: uri_str.to_string(),
            uri_part: UriPart::Username {
                user: "foo".to_string()
            }
        })
    }

    #[test]
    fn should_fail_base_uri_withpath() {
        let uri_str = "http://localhost:8080/info";
        let uri = URI::try_from(uri_str).unwrap();
        let result = verify_base_uri(&uri);

        assert_eq!(result.is_err(), true);

        let app_error = result.unwrap_err();

        assert_eq!(app_error.kind, AppConfigErrorKind::InvalidUriPart {
            base_uri: uri_str.to_string(),
            uri_part: UriPart::Path {
                path: "/info".to_string()
            }
        })
    }

    #[test]
    fn should_fail_base_uri_with_query() {
        let uri_str = "http://localhost:8080/?foo=bar";
        let uri = URI::try_from(uri_str).unwrap();
        let result = verify_base_uri(&uri);

        assert_eq!(result.is_err(), true);

        let app_error = result.unwrap_err();

        assert_eq!(app_error.kind, AppConfigErrorKind::InvalidUriPart {
            base_uri: uri_str.to_string(),
            uri_part: UriPart::Query {
                query: "foo=bar".to_string()
            }
        })
    }

    #[test]
    fn should_fail_base_uri_with_fragment() {
        let uri_str = "http://localhost:8080/#info";
        let uri = URI::try_from(uri_str).unwrap();
        let result = verify_base_uri(&uri);

        assert_eq!(result.is_err(), true);

        let app_error = result.unwrap_err();

        assert_eq!(app_error.kind, AppConfigErrorKind::InvalidUriPart {
            base_uri: uri_str.to_string(),
            uri_part: UriPart::Fragment {
                fragment: "info".to_string()
            }
        })
    }


    #[test]
    fn should_validate_valid_zone_name() {
        assert_eq!(verify_zone_name("ccsac.de"), Ok(()))
    }

    #[test]
    fn should_validate_valid_rev_zone_name() {
        assert_eq!(verify_zone_name("26.16.172.in-addr.arpa"), Ok(()))
    }

    #[test]
    fn should_fail_invalid_zone_name() {
        assert_eq!(verify_zone_name("ccsac").is_err(), true)
    }
}