use anyhow::Result;
use mass_rc::MaasClient;

fn main() -> Result<()> {
    let maas_url = "http://localhost:5240/MAAS";

    let api_key = "0e0cyQSfgRbvw0hN3x:DQtWtevINOd54JZJQ0:xzlD2jHLNxFH8mPoJhCWvbKOAb0ljvol";

    let client = MaasClient::new(maas_url, api_key, "2.0")?;

    println!("Connecting to MAAS at {}...", maas_url);

    let machines = client.get("/machines/")?;

    println!("Response: {}", serde_json::to_string_pretty(&machines)?);

    Ok(())
}