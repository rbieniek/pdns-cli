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
use log::{info, warn};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::oneshot::{Receiver, Sender};

use crate::pdns::error::Error;
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::rest_client::errors::RestClientError;

pub struct PowerDnsRestClient {
    request_builder: ClientRequestBuilder,
}

#[allow(dead_code)]
pub struct PnsServerResponse<I, O> where O: DeserializeOwned {
    request: I,
    response: Result<O, RestClientError>,
}

pub type PathProvider<I> = fn(&I) -> String;
pub type BodyProvider<I, T> = fn(&I) -> T;

impl PowerDnsRestClient {
    pub fn new(request_builder: ClientRequestBuilder) -> PowerDnsRestClient {
        PowerDnsRestClient {
            request_builder,
        }
    }

    pub async fn handle_get_request<I, O>(&self,
                                          request_rx: Receiver<I>,
                                          response_tx: Sender<PnsServerResponse<I, O>>,
                                          req_path_provider: PathProvider<I>) where O: DeserializeOwned {
        match request_rx.await {
            Ok(request_event) => {
                let mut request_path = "api/v1/".to_string();

                request_path.push_str(req_path_provider(&request_event).as_str());

                info!("Executing GET request to resource {}", &request_path);

                let result: Result<O, RestClientError> = match self.request_builder
                    .get_for_path(request_path.as_str())
                    .send()
                    .await {
                    Ok(rest_response) if is_success(rest_response.status()) => match rest_response.json::<O>().await {
                        Ok(server_response) => Ok(server_response),
                        Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                    },
                    Ok(rest_response) if is_known_error(rest_response.status()) => {
                        let status_code = rest_response.status();

                        match rest_response.json::<Error>().await {
                            Ok(server_response) => Err(RestClientError::on_powerdns_server_error(status_code, server_response)),
                            Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                        }
                    }
                    Ok(rest_response) => Err(RestClientError::on_client_error(rest_response.status())),
                    Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                };

                if let Err(_) = response_tx.send(PnsServerResponse::new(request_event, result)) {
                    warn!("Cannot send response");
                }
            }
            Err(error) => warn!("Expected message, didn't get one, error {}", error.to_string())
        }
    }

    pub async fn handle_post_request<I, O, T>(&self,
                                              request_rx: Receiver<I>,
                                              response_tx: Sender<PnsServerResponse<I, O>>,
                                              req_path_provider: PathProvider<I>,
                                              body_provider: BodyProvider<I, T>,
    ) where O: DeserializeOwned, T: Serialize {
        match request_rx.await {
            Ok(request_event) => {
                let mut request_path = "api/v1/".to_string();
                let payload: T = body_provider(&request_event);

                request_path.push_str(req_path_provider(&request_event).as_str());

                info!("Executing POST request to resource {} with payload {}",
                    &request_path,
                    serde_json::to_string(&payload).unwrap());

                let result: Result<O, RestClientError> = match self.request_builder
                    .post_for_path(request_path.as_str())
                    .json(&payload)
                    .send()
                    .await {
                    Ok(rest_response) if is_success(rest_response.status()) => match rest_response.json::<O>().await {
                        Ok(server_response) => Ok(server_response),
                        Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                    },
                    Ok(rest_response) if is_known_error(rest_response.status()) => {
                        let status_code = rest_response.status();

                        match rest_response.json::<Error>().await {
                            Ok(server_response) => Err(RestClientError::on_powerdns_server_error(status_code, server_response)),
                            Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                        }
                    }
                    Ok(rest_response) => Err(RestClientError::on_client_error(rest_response.status())),
                    Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                };

                if let Err(_) = response_tx.send(PnsServerResponse::new(request_event, result)) {
                    warn!("Cannot send response");
                }
            }
            Err(error) => warn!("Expected message, didn't get one, error {}", error.to_string())
        }
    }

    pub async fn handle_delete_request<I>(&self,
                                          request_rx: Receiver<I>,
                                          response_tx: Sender<PnsServerResponse<I, ()>>,
                                          req_path_provider: PathProvider<I>) {
        match request_rx.await {
            Ok(request_event) => {
                let mut request_path = "api/v1/".to_string();

                request_path.push_str(req_path_provider(&request_event).as_str());

                info!("Executing DELETE request to resource {}", &request_path);

                let result: Result<(), RestClientError> = match self.request_builder
                    .delete_for_path(request_path.as_str())
                    .send()
                    .await {
                    Ok(rest_response) if is_success(rest_response.status()) => Ok(()),
                    Ok(rest_response) if is_known_error(rest_response.status()) => {
                        let status_code = rest_response.status();

                        match rest_response.json::<Error>().await {
                            Ok(server_response) => Err(RestClientError::on_powerdns_server_error(status_code, server_response)),
                            Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                        }
                    }
                    Ok(rest_response) => Err(RestClientError::on_client_error(rest_response.status())),
                    Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                };

                if let Err(_) = response_tx.send(PnsServerResponse::new(request_event, result)) {
                    warn!("Cannot send response");
                }
            }
            Err(error) => warn!("Expected message, didn't get one, error {}", error.to_string())
        }
    }

    pub async fn handle_patch_request<I, T>(&self,
                                              request_rx: Receiver<I>,
                                              response_tx: Sender<PnsServerResponse<I, ()>>,
                                              req_path_provider: PathProvider<I>,
                                              body_provider: BodyProvider<I, T>,
    ) where T: Serialize {
        match request_rx.await {
            Ok(request_event) => {
                let mut request_path = "api/v1/".to_string();
                let payload: T = body_provider(&request_event);

                request_path.push_str(req_path_provider(&request_event).as_str());

                info!("Executing PATCH request to resource {} with payload {}",
                    &request_path,
                    serde_json::to_string(&payload).unwrap());

                let result: Result<(), RestClientError> = match self.request_builder
                    .patch_for_path(request_path.as_str())
                    .json(&payload)
                    .send()
                    .await {
                    Ok(rest_response) if is_success(rest_response.status()) => Ok(()),
                    Ok(rest_response) if is_known_error(rest_response.status()) => {
                        let status_code = rest_response.status();

                        match rest_response.json::<Error>().await {
                            Ok(server_response) => Err(RestClientError::on_powerdns_server_error(status_code, server_response)),
                            Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                        }
                    }
                    Ok(rest_response) => Err(RestClientError::on_client_error(rest_response.status())),
                    Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                };

                if let Err(_) = response_tx.send(PnsServerResponse::new(request_event, result)) {
                    warn!("Cannot send response");
                }
            }
            Err(error) => warn!("Expected message, didn't get one, error {}", error.to_string())
        }
    }
}

impl<I, O> PnsServerResponse<I, O> where O: DeserializeOwned {
    fn new(request: I, response: Result<O, RestClientError>) -> PnsServerResponse<I, O> {
        PnsServerResponse {
            request,
            response,
        }
    }

    #[allow(dead_code)]
    pub fn request(&self) -> &I {
        &self.request
    }

    pub fn response(&self) -> &Result<O, RestClientError> {
        &self.response
    }
}

fn is_known_error(status_code: StatusCode) -> bool {
    match status_code {
        StatusCode::BAD_REQUEST => true,
        StatusCode::NOT_FOUND => true,
        StatusCode::UNPROCESSABLE_ENTITY => true,
        StatusCode::INTERNAL_SERVER_ERROR => true,
        _ => false,
    }
}

fn is_success(status_code: StatusCode) -> bool {
    match status_code {
        StatusCode::OK => true,
        StatusCode::CREATED => true,
        StatusCode::NO_CONTENT => true,
        _ => false,
    }
}