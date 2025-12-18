use oauth1_request::{authorize, Credentials, HmacSha1, Token};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use anyhow::{Context, Result};
use reqwest::Method;
use serde_json::Value;

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

    fn request(&self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Value>{
        let url = format!("{}/api/{}{}", self.base_url, self.api_version, endpoint);
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
            let text= response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("MAAS Error {}: {}", status, text));
        }

        let json: Value = response.json().context("Failed to parse JSON response")?;

        Ok(json)
    }
}