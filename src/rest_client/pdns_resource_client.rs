use log::warn;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use tokio::sync::oneshot::{Receiver, Sender};

use crate::pdns::error::Error;
use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::rest_client::errors::RestClientError;

pub struct PowerDnsRestClient {
    request_builder: ClientRequestBuilder,
}

pub struct PnsServerResponse<I, O> where O: DeserializeOwned {
    request: I,
    response: Result<O, RestClientError>,
}

pub type PathProvider<I> = fn(&I) -> String;

impl PowerDnsRestClient {
    pub fn new(request_builder: ClientRequestBuilder) -> PowerDnsRestClient {
        PowerDnsRestClient {
            request_builder,
        }
    }

    pub async fn handle_request_event<I, O, F>(&self,
                                               request_rx: Receiver<I>,
                                               response_tx: Sender<PnsServerResponse<I, O>>,
                                               req_path_provider: PathProvider<I>) where O: DeserializeOwned {
        match request_rx.await {
            Ok(request_event) => {
                let mut request_path = "api/v1/".to_string();

                request_path.push_str(req_path_provider(&request_event).as_str());

                let result: Result<O, RestClientError> = match self.request_builder
                    .for_path(request_path.as_str())
                    .send()
                    .await {
                    Ok(rest_response) if is_success(rest_response.status()) => match rest_response.json::<O>().await {
                        Ok(server_response) => Ok(server_response),
                        Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                    },
                    Ok(rest_response) if is_known_error(rest_response.status()) => match rest_response.json::<Error>().await {
                        Ok(server_response) => Err(RestClientError::on_powerdns_server_error(server_response)),
                        Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                    },
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
        _ => false,
    }
}