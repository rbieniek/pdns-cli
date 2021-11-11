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
use tokio::sync::oneshot::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::pdns::server::Server;
use crate::rest_client::errors::RestClientError;
use crate::rest_client::lifecycle::Disposeable;
use crate::rest_client::client_request_builder::ClientRequestBuilder;

pub struct GetServerRequestEvent {
    base_uri: String,
    api_key: String,
    response_channel: Sender<GetServerResponseEvent>,
}

pub struct GetServerResponseEvent {
    result: Result<Server, RestClientError>,
}

pub struct ServerResourceClient {
    join_handles: Vec<JoinHandle<()>>,
}

impl GetServerRequestEvent {
    pub fn new(base_uri: String,
               api_key: String,
               response_channel: Sender<GetServerResponseEvent>) -> GetServerRequestEvent {
        GetServerRequestEvent {
            base_uri,
            api_key,
            response_channel,
        }
    }
}

impl GetServerResponseEvent {
    pub fn new(result: Result<Server, RestClientError>) -> GetServerResponseEvent {
        GetServerResponseEvent {
            result
        }
    }

    pub fn result(&self) -> Result<Server, RestClientError> {
        match &self.result {
            Ok(server) => Ok(server.clone()),
            Err(error) => Err(error.clone())
        }
    }
}

impl ServerResourceClient {
    pub fn new() -> ServerResourceClient {
        ServerResourceClient {
            join_handles: Vec::new(),
        }
    }

    pub fn create_channel(&mut self) -> Sender<GetServerRequestEvent> {
        let (event_tx, event_rx) = channel::<GetServerRequestEvent>();

        self.join_handles.push(tokio::spawn(handle_request_event(event_rx)));

        return event_tx;
    }
}

impl Disposeable for ServerResourceClient {
    fn shutdown(&self) {
        for handle in self.join_handles.iter() {
            handle.abort();
        }
    }
}

async fn handle_request_event(event_rx: Receiver<GetServerRequestEvent>) {
    match event_rx.await {
        Ok(request) => {
            info!("Received GetServerRequestEvent(baseUri={})", &request.base_uri);

            let result = match ClientRequestBuilder::new(&request.base_uri, &request.api_key)
                .get_for_path(&"api/v1/servers/localhost")
                .send()
                .await {
                Ok(rest_response) if rest_response.status() == StatusCode::OK => match rest_response.json::<Server>().await {
                    Ok(server_response) => Ok(server_response),
                    Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
                },
                Ok(rest_response) => Err(RestClientError::on_client_error(rest_response.status())),
                Err(rest_err) => Err(RestClientError::on_reqwest_runtime_error(rest_err.to_string())),
            };

            if let Err(_) = request.response_channel.send(GetServerResponseEvent::new(result)) {
                warn!("Cannot send response");
            }
        }
        Err(error) => info!("Expected message, didn't get one, error {}", error.to_string())
    }
}