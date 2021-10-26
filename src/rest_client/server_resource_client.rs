use crate::pdns::server::Server;
use crate::rest_client::errors::RestClientError;

pub struct GetServerRequestEvent {
    base_uri: String,
}

pub struct GetServerResponseEvent {
    result: Result<Server, RestClientError>
}

