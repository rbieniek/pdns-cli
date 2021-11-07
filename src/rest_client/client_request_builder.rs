use reqwest::header::{HeaderName, HeaderValue, ACCEPT, HeaderMap, CONTENT_TYPE, CACHE_CONTROL};
use reqwest::{Client, RequestBuilder};

pub struct ClientRequestBuilder {
    base_uri: String,
    api_key: String,
}

impl ClientRequestBuilder {
    pub fn new(base_uri: &String, api_key: &String) -> ClientRequestBuilder {
        ClientRequestBuilder {
            base_uri: base_uri.clone(),
            api_key: api_key.clone(),
        }
    }

    pub fn get_for_path(&self, path: &str) -> RequestBuilder {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        let mut request_uri = self.base_uri.clone();

        request_uri.push_str(path);
        headers.append(HeaderName::from_static("x-api-key"),
                       HeaderValue::from_str(&self.api_key.clone().as_str()).unwrap());
        headers.append(ACCEPT, HeaderValue::from_static("application/json"));
        headers.append(CACHE_CONTROL, HeaderValue::from_static("no-cache"));

        client.get(request_uri).headers(headers)
    }

    pub fn post_for_path(&self, path: &str) -> RequestBuilder {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        let mut request_uri = self.base_uri.clone();

        request_uri.push_str(path);
        headers.append(HeaderName::from_static("x-api-key"),
                       HeaderValue::from_str(&self.api_key.clone().as_str()).unwrap());
        headers.append(ACCEPT, HeaderValue::from_static("application/json"));
        headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.append(CACHE_CONTROL, HeaderValue::from_static("no-cache"));

        client.post(request_uri).headers(headers)
    }
}