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
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Eq)]
pub struct AppConfigError {
    pub(super) kind: AppConfigErrorKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppConfigErrorKind {
    MalformedBaseUri { base_uri: String, parser_error: String },
    InvalidUriPart { base_uri: String, uri_part: UriPart },
    MalformedZoneName { zone_name: String, reason: String },
    MalformedNumber { number: String },
    MissingCommand,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UriPart {
    Scheme { scheme: String },
    Username { user: String },
    Password { password: String },
    Path { path: String },
    Query { query: String },
    Fragment { fragment: String },
}


impl Error for AppConfigError {}

impl AppConfigError {
    pub fn on_malformed_base_uri(_base_uri: &String, _parser_error: &dyn Error) -> AppConfigError {
        AppConfigError {
            kind: AppConfigErrorKind::on_malformed_base_uri(_base_uri, &format!("{}", _parser_error))
        }
    }

    pub fn on_invalid_uri_part(_base_uri: &String, _uri_part: &UriPart) -> AppConfigError {
        AppConfigError {
            kind: AppConfigErrorKind::on_invalid_uri_part(_base_uri, _uri_part)
        }
    }

    pub fn on_malformed_zone_name(_zone_name: &String, _reason: &String) -> AppConfigError {
        AppConfigError {
            kind: AppConfigErrorKind::on_malformed_zone_name(_zone_name, _reason)
        }
    }

    pub fn on_missing_command() -> AppConfigError {
        AppConfigError {
            kind: AppConfigErrorKind::MissingCommand
        }
    }

    pub fn on_malformed_number(number: &String) -> AppConfigError {
        AppConfigError {
            kind: AppConfigErrorKind::on_malformed_number(number)
        }
    }

    fn __description(&self) -> String {
        match &self.kind {
            AppConfigErrorKind::MalformedBaseUri {
                base_uri,
                parser_error,
            } => format!("Malformed base URI '{}': {}", base_uri, parser_error),
            AppConfigErrorKind::InvalidUriPart {
                base_uri,
                uri_part,
            } => format!("Base URI '{}' contains unexpected part: {}", base_uri, uri_part),
            AppConfigErrorKind::MalformedZoneName {
                zone_name,
                reason,
            } => format!("Malformed zone name {}: {}", zone_name, reason),
            AppConfigErrorKind::MissingCommand => format!("Command missing"),
            AppConfigErrorKind::MalformedNumber { number} => format!("Malformed number: {}", number),
        }
    }
}

impl fmt::Display for AppConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.__description().fmt(f)
    }
}

impl AppConfigErrorKind {
    fn on_malformed_base_uri(_base_uri: &String, _parser_error: &String) -> AppConfigErrorKind {
        AppConfigErrorKind::MalformedBaseUri {
            base_uri: _base_uri.clone(),
            parser_error: _parser_error.clone(),
        }
    }

    fn on_invalid_uri_part(_base_uri: &String, _uri_part: &UriPart) -> AppConfigErrorKind {
        AppConfigErrorKind::InvalidUriPart {
            base_uri: _base_uri.clone(),
            uri_part: _uri_part.clone(),
        }
    }

    fn on_malformed_zone_name(_zone_name: &String, _reason: &String) -> AppConfigErrorKind {
        AppConfigErrorKind::MalformedZoneName {
            zone_name: _zone_name.clone(),
            reason: _reason.clone(),
        }
    }

    fn on_malformed_number(number: &String) -> AppConfigErrorKind {
        AppConfigErrorKind::MalformedNumber {
            number: number.clone(),
        }
    }
}

impl UriPart {
    fn __description(&self) -> String {
        match self {
            UriPart::Scheme { scheme } => format!("Invalid URI scheme '{}'", scheme),
            UriPart::Username { user } => format!("Non-Empty URI user name '{}'", user),
            UriPart::Password { password } => format!("Non-Empty URI password '{}'", password),
            UriPart::Path { path } => format!("Non-empty URI path '{}'", path),
            UriPart::Query { query } => format!("Non-empty query '{}'", query),
            UriPart::Fragment { fragment } => format!("Non-empty fragment '{}'", fragment),
        }
    }
}

impl fmt::Display for UriPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.__description().fmt(f)
    }
}