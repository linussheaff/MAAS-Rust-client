use anyhow::Result;
use mass_rs::MaasClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Get MAAS configuration from environment variables
    let maas_url = env::var("MAAS_URL")
        .unwrap_or_else(|_| "http://localhost:5240/MAAS".to_string());

    let api_key = env::var("MAAS_API_KEY")
        .expect("MAAS_API_KEY environment variable must be set");

    // Create the async client
    let client = MaasClient::new(&maas_url, &api_key, "2.0")?;

    println!("Connecting to MAAS at {}...", maas_url);

    // Fetch machines
    let machines = client.get("/machines/").await?;

    println!("Response: {}", serde_json::to_string_pretty(&machines)?);

    Ok(())
}