use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::mpsc::SendError;

use reqwest::StatusCode;

#[derive(Debug, PartialEq, Eq)]
pub struct RestClientError {
    pub(super) kind: RestClientErrorKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RestClientErrorKind {
    UnspecifiedError,
    TokioRuntimeError {
        tokio_error: String,
    },
    ReqwestRuntimeError {
        reqwest_error: String,
    },
    ClientError {
        status_code: StatusCode,
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

    pub fn on_reqwest_runtime_error(reqwest_error: String) -> RestClientError {
        RestClientError {
            kind: RestClientErrorKind::on_reqwest_runtime_error(reqwest_error),
        }
    }

    pub fn on_client_error(status_code: StatusCode) -> RestClientError {
        RestClientError {
            kind: RestClientErrorKind::on_client_error(status_code),
        }
    }

    fn __description(&self) -> String {
        match &self.kind {
            RestClientErrorKind::UnspecifiedError => format!("Unspecified error"),
            RestClientErrorKind::TokioRuntimeError { tokio_error } => format!("Tokio runtime error: {}", tokio_error),
            RestClientErrorKind::ReqwestRuntimeError { reqwest_error } => format!("Reqwest runtime error: {}", reqwest_error),
            RestClientErrorKind::ClientError { status_code } => format!("Unexpected client status code {}", status_code),
        }
    }
}

impl fmt::Display for RestClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.__description().fmt(f)
    }
}

impl RestClientErrorKind {
    fn on_unspecified_error() -> RestClientErrorKind {
        RestClientErrorKind::UnspecifiedError
    }

    fn on_tokio_runtime_error(tokio_error: String) -> RestClientErrorKind {
        RestClientErrorKind::TokioRuntimeError { tokio_error }
    }

    fn on_reqwest_runtime_error(reqwest_error: String) -> RestClientErrorKind {
        RestClientErrorKind::ReqwestRuntimeError { reqwest_error }
    }

    fn on_client_error(status_code: StatusCode) -> RestClientErrorKind {
        RestClientErrorKind::ClientError { status_code }
    }
}