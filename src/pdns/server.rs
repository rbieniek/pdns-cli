use serde::{Serialize, Deserialize};
use crate::pdns::struct_type::StructType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    #[serde(rename = "type")]
    type_id: StructType,
    id: String,
    daemon_type: DaemonType,
    version: String,
    url: String,
    config_url: String,
    zones_url: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DaemonType {
    #[serde(rename = "recursor")]
    Recursor,
    #[serde(rename = "authoritative")]
    Authoritative,
}

impl Server {
    pub fn type_id(&self) -> StructType {
        self.type_id.clone()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn daemon_type(&self) -> DaemonType {
        self.daemon_type.clone()
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn config_url(&self) -> String {
        self.config_url.clone()
    }

    pub fn zones_url(&self) -> String {
        self.zones_url.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::pdns::server::{Server, DaemonType};
    use crate::pdns::struct_type::StructType;

    #[test]
    fn should_deserialize_server_recursor() {
        let src = r#"
        {
            "type": "Server",
            "id": "localhost",
            "daemon_type": "recursor",
            "version": "1.0.0",
            "url": "http://localhost/",
            "config_url": "http://localhost/config",
            "zones_url": "http://localhost/zones"
        }
        "#;

        let server: Server = serde_json::from_str(src).unwrap();

        assert_eq!(server.type_id(), StructType::Server);
        assert_eq!(server.id(), "localhost".to_string());
        assert_eq!(server.daemon_type(), DaemonType::Recursor);
        assert_eq!(server.version(), "1.0.0");
        assert_eq!(server.url(), "http://localhost/".to_string());
        assert_eq!(server.config_url(), "http://localhost/config".to_string());
        assert_eq!(server.zones_url(), "http://localhost/zones".to_string());
    }

    #[test]
    fn should_deserialize_server_authoritative() {
        let src = r#"
        {
            "type": "Server",
            "id": "localhost",
            "daemon_type": "authoritative",
            "version": "1.0.0",
            "url": "http://localhost/",
            "config_url": "http://localhost/config",
            "zones_url": "http://localhost/zones"
        }
        "#;

        let server: Server = serde_json::from_str(src).unwrap();

        assert_eq!(server.type_id(), StructType::Server);
        assert_eq!(server.id(), "localhost".to_string());
        assert_eq!(server.daemon_type(), DaemonType::Authoritative);
        assert_eq!(server.version(), "1.0.0");
        assert_eq!(server.url(), "http://localhost/".to_string());
        assert_eq!(server.config_url(), "http://localhost/config".to_string());
        assert_eq!(server.zones_url(), "http://localhost/zones".to_string());
    }
}