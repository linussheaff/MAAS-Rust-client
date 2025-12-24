use serde::{Deserialize, Serialize};

/// Represents a physical or virtual machine managed by MAAS.
///
/// This matches the JSON output from `GET /api/2.0/machines/`.
/// We use `#[serde(rename_all = "snake_case")]` handling implicitly,
/// but explicit renames are added for clarity on critical fields.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Machine {
    /// The unique 5-character ID (e.g. "x8d7f").
    #[serde(rename = "system_id")]
    pub system_id: String,

    /// The user-defined hostname (e.g. "web-server-01").
    pub hostname: String,

    /// The current power state (e.g. "on", "off", "error").
    #[serde(rename = "power_state")]
    pub power_state: String,

    /// The architecture type (e.g. "amd64/generic").
    pub architecture: String,

    /// Total RAM in MiB.
    pub memory: u64,

    /// Total number of CPU cores detected.
    #[serde(rename = "cpu_count")]
    pub cpu_count: u32,

    /// The human-readable status name (e.g. "Ready", "Deployed", "Broken").
    #[serde(rename = "status_name")]
    pub status: String,

    /// List of IP addresses currently assigned to this machine.
    #[serde(rename = "ip_addresses", default)]
    pub ip_addresses: Vec<String>,

    /// Tags used for categorization (e.g. "virtual", "gpu-node").
    #[serde(rename = "tag_names", default)]
    pub tags: Vec<String>,
}

impl Machine {
    /// Returns true if the machine reports it is powered on.
    pub fn is_on(&self) -> bool {
        self.power_state == "on"
    }

    /// Returns a summary string useful for CLI output.
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} ({} cores, {} MiB) - {}",
            self.system_id, self.hostname, self.cpu_count, self.memory, self.status
        )
    }
}