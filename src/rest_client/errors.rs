
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
use std::fmt;

use reqwest::StatusCode;

use crate::pdns::error::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestClientError {
    pub(super) kind: RestClientErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestClientErrorKind {
    UnspecifiedError {
        message: Option<String>,
    },
    TokioRuntimeError {
        tokio_error: String,
    },
    ReqwestRuntimeError {
        reqwest_error: String,
    },
    ClientError {
        status_code: StatusCode,
    },
    PowerDnsServerError {
        status_code: StatusCode,
        server_error: Error,
    },
}

impl RestClientError {
    pub fn on_unspecified_error() -> RestClientError {
        RestClientError {
            kind: RestClientErrorKind::on_unspecified_error(),
        }
    }

    pub fn on_unspecified_error_message(message: &String) -> RestClientError {
        RestClientError {
            kind: RestClientErrorKind::on_unspecified_error_message(message),
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

    pub fn on_powerdns_server_error(status_code: StatusCode, server_error: Error) -> RestClientError {
        RestClientError {
            kind: RestClientErrorKind::on_powerdns_server_error(status_code, server_error),
        }
    }

    pub fn kind(&self) -> RestClientErrorKind {
        self.kind.clone()
    }

    fn __description(&self) -> String {
        match &self.kind {
            RestClientErrorKind::UnspecifiedError {
                message
            }  => format!("Unspecified error: {}", &message.clone().unwrap_or("None".to_string())),
            RestClientErrorKind::TokioRuntimeError {
                tokio_error
            } => format!("Tokio runtime error: {}", tokio_error),
            RestClientErrorKind::ReqwestRuntimeError {
                reqwest_error
            } => format!("Reqwest runtime error: {}", reqwest_error),
            RestClientErrorKind::ClientError {
                status_code
            } => format!("Unexpected client status code {}", status_code),
            RestClientErrorKind::PowerDnsServerError {
                server_error,
                status_code,
            } => format!("PowerDNS server error: status code: {}, server error {}",
                         status_code,
                         server_error),
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
        RestClientErrorKind::UnspecifiedError {
            message: None,
        }
    }

    fn on_unspecified_error_message(message: &String) -> RestClientErrorKind {
        RestClientErrorKind::UnspecifiedError {
            message: Some(message.clone()),
        }
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

    fn on_powerdns_server_error(status_code: StatusCode, server_error: Error) -> RestClientErrorKind {
        RestClientErrorKind::PowerDnsServerError {
            status_code,
            server_error,
        }
    }
}