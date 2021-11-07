use std::convert::TryFrom;

use clap::{App, Arg, ArgGroup, ArgMatches};
use fancy_regex::Regex;
use log::LevelFilter;
use uriparse::{Scheme, URI};

use crate::app_config::errors::{AppConfigError, UriPart};

const PARAM_BASE_URI: &'static str = "base-uri";
const PARAM_API_KEY: &'static str = "api-key";
const PARAM_ZONE_NAME: &'static str = "zone-name";
const PARAM_IGNORE_EXISTING: &'static str = "ignore-existing";
const PARAM_VERBOSITY: &'static str = "verbose";
const PARAM_REFRESH_TIME: &'static str = "refresh-time";
const PARAM_RETRY_TIME: &'static str = "retry-time";
const PARAM_EXPIRE_TIME: &'static str = "expire-time";
const PARAM_NEG_CACHE_TIME: &'static str = "negative-cache-time";
const PARAM_NAMESERVER: &'static str = "nameserver";
const PARAM_MASTER: &'static str = "master";
const PARAM_ACCOUNT: &'static str = "account";
const SUBCOMMAND_ADD_ZONE: &'static str = "add-zone";
const SUBCOMMAND_REMOVE_ZONE: &'static str = "remove-zone";
const SUBCOMMAND_QUERY_ZONE: &'static str = "query-zone";
const GROUP_NAMESERVER_OR_MASTER: &'static str = "nameserver-or-master";


pub struct ApplicationConfiguration {
    base_uri: String,
    api_key: String,
    log_level: LevelFilter,
    zone_name: String,
    command: Command,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub struct Command {
    kind: CommandKind,
    parameters: CommandParameters,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub enum CommandParameters {
    AddZone {
        refresh: u32,
        retry: u32,
        expire: u32,
        neg_caching: u32,
        masters: Vec<String>,
        nameservers: Vec<String>,
        account: String,
    },
    RemoveZone {
    },
    QueryZone {
    },
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub enum CommandKind {
    AddZone,
    RemoveZone,
    QueryZone,
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
            Some(Command {
                kind: CommandKind::AddZone,
                parameters: CommandParameters::AddZone {
                    refresh: arg_u32(&command, PARAM_REFRESH_TIME).unwrap_or(3600),
                    retry: arg_u32(&command, PARAM_RETRY_TIME).unwrap_or(3600),
                    expire: arg_u32(&command, PARAM_EXPIRE_TIME).unwrap_or(3600),
                    neg_caching: arg_u32(&command, PARAM_NEG_CACHE_TIME).unwrap_or(3600),
                    masters: arg_str_vec(&command, PARAM_MASTER),
                    nameservers: arg_str_vec(&command, PARAM_NAMESERVER),
                    account: command.value_of(PARAM_ACCOUNT).unwrap_or("root").to_string(),
                },
            })
        } else { None };
        let command_query_zone = if let Some(_) = matches.subcommand_matches(SUBCOMMAND_QUERY_ZONE) {
            Some(Command {
                kind: CommandKind::QueryZone,
                parameters: CommandParameters::QueryZone {},
            })
        } else { None };
        let command_remove_zone = if let Some(_) = matches.subcommand_matches(SUBCOMMAND_REMOVE_ZONE) {
            Some(Command {
                kind: CommandKind::RemoveZone,
                parameters: CommandParameters::RemoveZone {},
            })
        } else { None };

        match command_add_zone.or(command_query_zone).or(command_remove_zone) {
            Some(command) => Ok(ApplicationConfiguration {
                zone_name: matches.value_of(PARAM_ZONE_NAME).unwrap().to_string(),
                base_uri: matches.value_of(PARAM_BASE_URI).unwrap().to_string(),
                api_key: matches.value_of(PARAM_API_KEY).unwrap().to_string(),
                log_level: level,
                command,
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

    pub fn zone_name(&self) -> String {
        self.zone_name.clone()
    }

    pub fn command(&self) -> Command {
        self.command.clone()
    }

    pub fn log_level(&self) -> LevelFilter {
        self.log_level
    }
}

impl Command {
    pub fn kind(&self) -> CommandKind {
        self.kind.clone()
    }

    pub fn parameters(&self) -> CommandParameters {
        self.parameters.clone()
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
            .long(PARAM_API_KEY)
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
        .arg(Arg::new(PARAM_ZONE_NAME)
            .about("Zone name")
            .long(PARAM_ZONE_NAME)
            .short('n')
            .takes_value(true)
            .required(true)
            .validator(|value| verify_zone_name(value)))
        .subcommand(App::new(SUBCOMMAND_ADD_ZONE)
            .about("Add zone to PowerDNS instance")
            .group(ArgGroup::new(GROUP_NAMESERVER_OR_MASTER)
                .required(true)
                .multiple(true)
                .arg(PARAM_MASTER)
                .arg(PARAM_NAMESERVER))
            .arg(Arg::new(PARAM_REFRESH_TIME)
                .about("Refresh time")
                .long(PARAM_REFRESH_TIME)
                .required(false)
                .takes_value(true)
                .validator(|value| is_u32(value)))
            .arg(Arg::new(PARAM_RETRY_TIME)
                .about("Refresh time")
                .long(PARAM_RETRY_TIME)
                .required(false)
                .takes_value(true)
                .validator(|value| is_u32(value)))
            .arg(Arg::new(PARAM_EXPIRE_TIME)
                .about("Refresh time")
                .long(PARAM_EXPIRE_TIME)
                .required(false)
                .takes_value(true)
                .validator(|value| is_u32(value)))
            .arg(Arg::new(PARAM_NEG_CACHE_TIME)
                .about("Refresh time")
                .long(PARAM_NEG_CACHE_TIME)
                .required(false)
                .takes_value(true)
                .validator(|value| is_u32(value)))
            .arg(Arg::new(PARAM_ACCOUNT)
                .about("DNS admin account")
                .long(PARAM_ACCOUNT)
                .required(false)
                .takes_value(true))
            .arg(Arg::new(PARAM_MASTER)
                .about("Zone master, implies zone type slave")
                .long(PARAM_MASTER)
                .short('m')
                .required(false)
                .takes_value(true)
                .conflicts_with(PARAM_NAMESERVER)
                .multiple_occurrences(true))
            .arg(Arg::new(PARAM_NAMESERVER)
                .about("Zone master, implies zone type master")
                .long(PARAM_NAMESERVER)
                .short('n')
                .required(false)
                .takes_value(true)
                .conflicts_with(PARAM_MASTER)
                .multiple_occurrences(true)))
        .subcommand(App::new(SUBCOMMAND_QUERY_ZONE)
            .about("Add zone to PowerDNS instance"))
        .subcommand(App::new(SUBCOMMAND_REMOVE_ZONE)
            .about("Add zone to PowerDNS instance"))
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

fn arg_u32(command: &ArgMatches, name: &'static str) -> Option<u32> {
    match command.value_of(name) {
        Some(value) => match value.parse::<u32>() {
            Ok(number) if number > 0 => Some(number),
            Ok(_) => None,
            Err(_) => None,
        }
        None => None,
    }
}

fn is_u32(value: &str) -> Result<(), AppConfigError> {
    match value.parse::<u32>() {
        Ok(number) if number > 0 => Ok(()),
        _ => Err(AppConfigError::on_malformed_number(&value.to_string())),
    }
}

fn arg_str_vec(command: &ArgMatches, name: &'static str) -> Vec<String> {
    match command.values_of(name) {
        Some(values) => {
            let mut args: Vec<String> = Vec::new();

            for value in values.into_iter() {
                args.push(value.to_string())
            }

            args
        }
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use uriparse::URI;

    use crate::app_config::cmd_line_parser::{verify_base_uri, verify_zone_name};
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
            },
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
            },
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
            },
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
            },
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
            },
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