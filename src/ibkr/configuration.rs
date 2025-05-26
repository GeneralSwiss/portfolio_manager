// -----------------------------------------------------------------------------
// Interactive Brokers client configuration.
// Start small: a value object that can be deserialized from `TOML`, YAML, or env vars
// and passed into the eventual `IbClient` connector. No network code yet—this file
// just defines *how* we describe where TWS/Gateway lives and what logical client ID
// we will use.
// -----------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

/// Connection settings for TWS or IB Gateway.
///
/// You can instantiate this manually:
/// ```no_compile
/// let cfg = IbClientConfig::new("127.0.0.1", 4002, 0);
/// ```
/// or load it from a `config.toml` using `serde` + `toml` crate:
/// ```toml
/// host      = "127.0.0.1"
/// port      = 4002        # paper trading in TWS
/// client_id = 0           # must be unique per connection
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IbClientConfig {
    /// Gateway/TWS host (usually 127.0.0.1 when running locally)
    pub host: String,
    /// Port: 4001 = TWS live, 4002 = TWS paper, 7497 = Gateway live, 4003 = Gateway paper
    pub port: u16,
    /// Unique ID so TWS can distinguish multiple API clients.
    pub client_id: i32,
}

impl Default for IbClientConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 4001, // sensible default: TWS paper
            client_id: 0,
        }
    }
}

impl IbClientConfig {
    /// Shorthand constructor.
    pub fn new<H: Into<String>>(host: H, port: u16, client_id: i32) -> Self {
        Self {
            host: host.into(),
            port,
            client_id,
        }
    }

    pub fn get_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configuration() {
        // Test default method for creating a default configuration.
        let config = IbClientConfig::default();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 4002); // Paper trading default port
        assert_eq!(config.client_id, 0);
    }

    #[test]
    fn test_new_configuration() {
        // Test new method for creating a custom configuration.
        let config = IbClientConfig::new("192.168.1.10", 1234, 42);

        assert_eq!(config.host, "192.168.1.10");
        assert_eq!(config.port, 1234);
        assert_eq!(config.client_id, 42);
    }

    #[test]
    fn test_get_address() {
        // Ensure the get_address method returns the correct formatted string.
        let config = IbClientConfig::new("10.0.0.1", 8080, 1);

        assert_eq!(config.get_address(), "10.0.0.1:8080");
    }

    #[test]
    fn test_eq_for_custom_configs() {
        // Verify equality comparisons if the struct behavior changes in future
        let config1 = IbClientConfig::new("localhost", 8000, 10);
        let config2 = IbClientConfig::new("localhost", 8000, 10);

        assert_eq!(config1.host, config2.host);
        assert_eq!(config1.port, config2.port);
        assert_eq!(config1.client_id, config2.client_id);
    }

    #[test]
    fn test_clone_configuration() {
        // Ensure that cloning works correctly with no data loss.
        let config = IbClientConfig::new("127.0.0.1", 5000, 2);
        let cloned_config = config.clone();

        assert_eq!(config.host, cloned_config.host);
        assert_eq!(config.port, cloned_config.port);
        assert_eq!(config.client_id, cloned_config.client_id);
    }

    #[test]
    fn test_serde_serialization() {
        // Test serializing to JSON
        let config = IbClientConfig::new("127.0.0.1", 4002, 3);
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"host\":\"127.0.0.1\""));
        assert!(json.contains("\"port\":4002"));
        assert!(json.contains("\"client_id\":3"));
    }

    #[test]
    fn test_serde_deserialization() {
        // Test deserializing from JSON
        let json = r#"
        {
            "host": "10.0.0.1",
            "port": 8080,
            "client_id": 7
        }
        "#;
        let config: IbClientConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.host, "10.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.client_id, 7);
    }

    #[test]
    fn test_invalid_json_deserialization() {
        // Test for handling invalid JSON cases
        let invalid_json = r#"
        {
            "host": "127.0.0.1",
            "port": "not_a_number",
            "client_id": 0
        }
        "#;
        let result: Result<IbClientConfig, serde_json::Error> = serde_json::from_str(invalid_json);

        assert!(result.is_err(), "Invalid JSON should fail deserialization");
    }
}
