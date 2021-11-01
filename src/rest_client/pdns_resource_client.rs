use log::warn;

use crate::rest_client::client_request_builder::ClientRequestBuilder;
use crate::pdns::common::PowerDnsPayload;
use crate::rest_client::errors::RestClientError;
use tokio::sync::oneshot::{Sender, Receiver};
use reqwest::StatusCode;
use crate::pdns::error::Error;
use serde::de::DeserializeOwned;

pub struct PowerDnsRestClient {
    request_builder: ClientRequestBuilder,
}

pub trait RequestPathProvider {
    fn provide_request_path(&self) -> String;
}

pub trait RequestEvent<Payload: PowerDnsPayload> {
    fn response(&self, result: Result<Payload, RestClientError>) -> Box<dyn ResponseEvent<Payload>>;

    fn response_channel(&self) -> Sender<Box<dyn ResponseEvent<Payload>>>;

    fn request_path_provider(&self) -> Box<dyn RequestPathProvider>;
}

pub trait ResponseEvent<T: PowerDnsPayload> {
    fn result(&self) -> Result<T, RestClientError>;
}

impl PowerDnsRestClient {
    pub fn new(request_builder: ClientRequestBuilder) -> PowerDnsRestClient {
        PowerDnsRestClient {
            request_builder,
        }
    }

    pub async fn handle_request_event<Payload: PowerDnsPayload + DeserializeOwned>(&self, event_rx: Receiver<Box<dyn RequestEvent<Payload>>>) {
        match event_rx.await {
            Ok(request_event) => {
                let mut request_path = "api/v1/".to_string();

                request_path.push_str(&request_event.request_path_provider().provide_request_path());

                let result: Result<Payload, RestClientError> = match self.request_builder
                    .for_path(request_path.as_str())
                    .send()
                    .await {
                    Ok(rest_response) if is_success(rest_response.status()) => match rest_response.json::<Payload>().await {
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

                if let Err(_) = request_event.response_channel().send(request_event.response(result)) {
                    warn!("Cannot send response");
                }
            }
            Err(error) => warn!("Expected message, didn't get one, error {}", error.to_string())
        }
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