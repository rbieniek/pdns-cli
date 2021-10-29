use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::mpsc::SendError;

#[derive(Debug, PartialEq, Eq)]
pub struct RestClientError {
    pub(super) kind: RestClientErrorKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RestClientErrorKind {
    UnspecifiedError,
    TokioRuntimeError {
        tokio_error : String,
    },
}

impl RestClientError {
    pub fn on_unspecified_error() -> RestClientError {
        RestClientError {
            kind: RestClientErrorKind::on_unspecified_error(),
        }
    }

    pub fn on_tokio_runtime_error(tokio_error: String) -> RestClientError {
        RestClientError {
            kind: RestClientErrorKind::on_tokio_runtime_error(tokio_error),
        }
    }

    fn __description(&self) -> String {
        match &self.kind {
            RestClientErrorKind::UnspecifiedError => format!("Unspecified error"),
            RestClientErrorKind::TokioRuntimeError { tokio_error } => format!("Tokio runtime error: {}", tokio_error),
        }
    }
}

impl fmt::Display for RestClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.__description().fmt(f)
    }
}

impl  RestClientErrorKind {
    fn on_unspecified_error() -> RestClientErrorKind {
        RestClientErrorKind::UnspecifiedError
    }

    fn on_tokio_runtime_error(tokio_error : String) -> RestClientErrorKind {
        RestClientErrorKind::TokioRuntimeError { tokio_error }
    }
}