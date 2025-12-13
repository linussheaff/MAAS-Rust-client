use anyhow::{Context, Result};
use oauth1_request::{authorize, Credentials, Token, signature_method::HmacSha1};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

// Client struct
pub struct MaasClient {
    base_url: String,
    consumer_key: String,
    token_key: String,
    token_secret: String,
    client: reqwest::blocking::Client,
}

impl MaasClient {
    pub fn new(base_url: &str, api_key: &str) -> Result<Self> {
        let parts: Vec<&str> = api_key.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid API Key format. Expected A:B:C"));
        }

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            consumer_key: parts[0].to_string(),
            token_key: parts[1].to_string(),
            token_secret: parts[2].to_string(),
            client: reqwest::blocking::Client::new(),
        })
    }

    fn generate_auth_header(&self, method: &str, url: &str) -> String {
        // MAAS consumer secret
        let client_creds = Credentials::new(self.consumer_key.as_str(), "");

        let token_creds = Credentials::new(self.token_key.as_str(), self.token_secret.as_str());

        let oauth_token = Token::new(client_creds, token_creds);

        authorize(
            method,
            url,
            &(),
            &oauth_token,
            HmacSha1,
        )
    }

    pub fn get(&self, endpoint: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/2.0{}", self.base_url, endpoint);

        let auth_header = self.generate_auth_header("GET", &url);

        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, auth_header)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("MAAS Error {}: {}", status, text));
        }

        let json: serde_json::Value = response.json()?;
        Ok(json)
    }
}

fn main() -> Result<()> {
    let maas_url = "http://localhost:5240/MAAS";

    let api_key = "";

    let client = MaasClient::new(maas_url, api_key)?;

    println!("Connecting to MAAS at {}...", maas_url);

    let machines = client.get("/machines/")?;

    println!("Response: {}", serde_json::to_string_pretty(&machines)?);

    Ok(())
}