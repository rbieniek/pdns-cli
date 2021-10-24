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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UriPart {
    Scheme { scheme: String },
    Path { path: String },
    Query { query: String },
    Anchor { anchor: String },
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
}

impl UriPart {
    fn __description(&self) -> String {
        match self {
            UriPart::Scheme { scheme} => format!("Invalid URI scheme '{}'", scheme),
            UriPart::Path { path } => format!("Non-empty URI path '{}'", path),
            UriPart::Query { query} => format!("Non-empty query '{}'", query),
            UriPart::Anchor { anchor } => format!("Non-empty anchor '{}'", anchor),
        }
    }
}

impl fmt::Display for UriPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.__description().fmt(f)
    }
}