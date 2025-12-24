use crate::error::MaasError;

use anyhow::{Context};
use oauth1_request::{authorize, signature_method::HmacSha1, Credentials, Token};
use serde_json::Value;


// Default async client
#[cfg(feature = "async")]
pub mod client{
    use super::*;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

    #[derive(Debug)]
    pub struct MaasClient {
        pub(crate) base_url: String,
        consumer_key: String,
        token_key: String,
        token_secret: String,
        pub(crate) api_version: String,
        client: reqwest::Client,
    }

    impl MaasClient {
        pub fn new(base_url: &str, api_key: &str, api_version: &str) -> anyhow::Result<Self, MaasError> {
            let parts: Vec<&str> = api_key.split(':').collect();
            if parts.len() != 3 {
                return Err(MaasError::InvalidKeyFormat);
            }

            Ok(Self {
                base_url: base_url.trim_end_matches('/').to_string(),
                consumer_key: parts[0].to_string(),
                token_key: parts[1].to_string(),
                token_secret: parts[2].to_string(),
                api_version: api_version.to_string(),
                client: reqwest::Client::new(),
            })
        }

        /// Generates the Oauth1 header for the request
        pub(crate) fn generate_auth_header(&self, method: &str, url: &str) -> String {
            // MAAS consumer secret is empty
            let client_creds = Credentials::new(self.consumer_key.as_str(), "");
            let token_creds = Credentials::new(self.token_key.as_str(), self.token_secret.as_str());
            let oauth_token = Token::new(client_creds, token_creds);

            authorize(method, url, &(), &oauth_token, HmacSha1)
        }

        pub async fn get(&self, endpoint: &str) -> anyhow::Result<Value, MaasError> {
            self.request("GET", endpoint, None).await
        }

        pub async fn post(&self, endpoint: &str, body: Option<Value>) -> anyhow::Result<Value, MaasError> {
            self.request("POST", endpoint, body).await
        }

        pub async fn put(&self, endpoint: &str, body: Option<Value>) -> anyhow::Result<Value, MaasError> {
            self.request("PUT", endpoint, body).await
        }

        pub async fn delete(&self, endpoint: &str) -> anyhow::Result<Value, MaasError> {
            self.request("DELETE", endpoint, None).await
        }

        ///Performs HTTP requests to MAAS API
        async fn request(&self, method: &str, endpoint: &str, body: Option<Value>) -> anyhow::Result<Value, MaasError> {
            // authenticate and build request
            let url = format!("{}/api/{}/{}", self.base_url, self.api_version, endpoint.trim_start_matches('/'));
            let auth_header = self.generate_auth_header(method, &url);
            let mut request = self.client.request(
                method.parse().unwrap_or(reqwest::Method::GET),
                &url,
            );
            request = request
                .header(AUTHORIZATION, auth_header)
                .header(CONTENT_TYPE, "application/json");

            if let Some(body) = body {
                request = request.json(&body);
            }

            let response = request.send().await?;

            let status = response.status();

            if !status.is_success() {
                let text = response.text().await.unwrap_or_default();
                return Err(MaasError::ApiError {status, body: text});
            }

            let json: Value = response.json().await?;

            Ok(json)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "async")]
    use client::MaasClient;
    #[cfg(all(feature = "blocking", not(feature = "async")))]
    use blocking_client::MaasClient;

    // Constructors

    #[test]
    fn test_new_client_valid_key() {
        let result = MaasClient::new("http://localhost:5240/MAAS", "key:token:secret", "2.0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_client_invalid_key_format() {
        // Too few parts
        let result = MaasClient::new("http://localhost:5240/MAAS", "part1:part2", "2.0");
        match result{
            Err(MaasError::InvalidKeyFormat) => assert!(true),
            _ => panic!("Expected InvalidKeyFormat error, instead got {:?}", result),
        }

        // Too many parts
        let result = MaasClient::new("http://localhost:5240/MAAS", "a:b:c:d", "2.0");
        assert!(matches!(result, Err(MaasError::InvalidKeyFormat)));
    }

    #[test]
    fn test_new_client_empty_parts() {
        // Empty MAAS keys
        let result = MaasClient::new("http://localhost:5240/MAAS", "::", "2.0");
        assert!(result.is_ok());
    }

    // URL

    #[test]
    fn test_url_trimming_logic() {
        // trailing slashes on base_url
        let client = MaasClient::new("http://maas.local/", "a:b:c", "2.0").unwrap();
        assert_eq!(client.base_url, "http://maas.local");

        // multiple trailing slashes
        let client = MaasClient::new("http://maas.local///", "a:b:c", "2.0").unwrap();
        assert_eq!(client.base_url, "http://maas.local");
    }

    #[test]
    fn test_api_version_assignment() {
        let client = MaasClient::new("http://localhost", "a:b:c", "v3").unwrap();
        assert_eq!(client.api_version, "v3");
    }

    // OAuth

    #[test]
    fn test_oauth_header_generation_consistency() {
        // contains key
        let client = MaasClient::new("http://localhost", "cons:tok:sec", "2.0").unwrap();
        let url = "http://localhost/api/2.0/machines/";

        let header = client.generate_auth_header("GET", url);

        assert!(header.contains("oauth_consumer_key=\"cons\""));
        assert!(header.contains("oauth_token=\"tok\""));
        assert!(header.contains("oauth_signature_method=\"HMAC-SHA1\""));
        assert!(header.contains("oauth_signature="));
    }

    #[test]
    fn test_endpoint_slash_handling() {
        // When the user provides "machines/" or "/machines/", the URL shouldn't change

        let base = "http://localhost";
        let ver = "2.0";
        let e1 = "machines/";
        let e2 = "/machines/";

        let url1 = format!("{}/api/{}/{}", base, ver, e1.trim_start_matches('/'));
        let url2 = format!("{}/api/{}/{}", base, ver, e2.trim_start_matches('/'));

        assert_eq!(url1, "http://localhost/api/2.0/machines/");
        assert_eq!(url1, url2);
    }
}



///Build the non-async client
#[cfg(feature="blocking")]
pub mod blocking_client {
    use super::*;


    pub struct MaasClient {
        base_url: String,
        consumer_key: String,
        token_key: String,
        token_secret: String,
        api_version: String,
        client: reqwest::blocking::Client,
    }

    impl MaasClient {
        pub fn new(base_url: &str, api_key: &str, api_version: &str) -> anyhow::Result<Self> {
            let parts: Vec<&str> = api_key.split(':').collect();
            if parts.len() != 3 {
                return Err(anyhow::anyhow!("Invalid API Key format. Expected A:B:C"));
            }

            Ok(Self {
                base_url: base_url.trim_end_matches('/').to_string(),
                consumer_key: parts[0].to_string(),
                token_key: parts[1].to_string(),
                token_secret: parts[2].to_string(),
                api_version: api_version.to_string(),
                client: reqwest::blocking::Client::new(),
            })
        }

        fn generate_auth_header(&self, method: &str, url: &str) -> String {
            // MAAS consumer secret is empty
            let client_creds = Credentials::new(self.consumer_key.as_str(), "");
            let token_creds = Credentials::new(self.token_key.as_str(), self.token_secret.as_str());
            let oauth_token = Token::new(client_creds, token_creds);

            authorize(method, url, &(), &oauth_token, HmacSha1)
        }

        pub fn get(&self, endpoint: &str) -> anyhow::Result<serde_json::Value> {
            self.request("GET", endpoint, None)
        }

        pub fn post(&self, endpoint: &str, body: Option<Value>) -> anyhow::Result<serde_json::Value> {
            self.request("POST", endpoint, body)
        }

        pub fn put(&self, endpoint: &str, body: Option<Value>) -> anyhow::Result<serde_json::Value> {
            self.request("PUT", endpoint, body)
        }

        pub fn delete(&self, endpoint: &str) -> anyhow::Result<serde_json::Value> {
            self.request("DELETE", endpoint, None)
        }

        fn request(&self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Value> {
            let url = format!("{}/api/{}/{}", self.base_url, self.api_version, endpoint.trim_start_matches('/'));
            let auth_header = self.generate_auth_header(method, &url);
            let mut request = self.client.request(
                method.parse().context("Invalid HTTP method")?,
                &url,
            );
            request = request
                .header(AUTHORIZATION, auth_header)
                .header(CONTENT_TYPE, "application/json");

            if let Some(body) = body {
                request = request.json(&body);
            }

            let response = request.send().context("Failed to send request")?;

            let status = response.status();
            if !status.is_success() {
                let text = response.text().unwrap_or_default();
                return Err(anyhow::anyhow!("MAAS Error {}: {}", status, text));
            }

            let json: Value = response.json().context("Failed to parse JSON response")?;

            Ok(json)
        }
    }
}