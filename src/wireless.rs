use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::transport::UsbDeviceInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairingRecord {
    pub device_id: String,
    pub host_id: String,
    pub reconnect_token: String,
    pub last_seen_unix: u64,
}

#[derive(Debug, Default)]
pub struct PairingStore {
    records: Vec<PairingRecord>,
}

impl PairingStore {
    pub fn load(path: &Path) -> Result<Self, WirelessError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).map_err(WirelessError::Io)?;
        let mut store = Self::default();
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() != 4 {
                return Err(WirelessError::CorruptStore);
            }
            let last_seen_unix = parts[3]
                .parse::<u64>()
                .map_err(|_| WirelessError::CorruptStore)?;
            store.records.push(PairingRecord {
                device_id: parts[0].to_string(),
                host_id: parts[1].to_string(),
                reconnect_token: parts[2].to_string(),
                last_seen_unix,
            });
        }

        Ok(store)
    }

    pub fn save(&self, path: &Path) -> Result<(), WirelessError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(WirelessError::Io)?;
            }
        }

        let mut output = String::new();
        for record in &self.records {
            output.push_str(&format!(
                "{}|{}|{}|{}\n",
                record.device_id, record.host_id, record.reconnect_token, record.last_seen_unix
            ));
        }
        fs::write(path, output).map_err(WirelessError::Io)
    }

    pub fn upsert(&mut self, record: PairingRecord) {
        if let Some(existing) = self
            .records
            .iter_mut()
            .find(|candidate| candidate.device_id == record.device_id)
        {
            *existing = record;
        } else {
            self.records.push(record);
        }
    }

    pub fn find(&self, device_id: &str) -> Option<&PairingRecord> {
        self.records.iter().find(|record| record.device_id == device_id)
    }
}

#[derive(Debug, Default)]
pub struct WirelessDiscovery;

impl WirelessDiscovery {
    pub fn discover_candidate(&self, usb_devices: &[UsbDeviceInfo]) -> Option<String> {
        usb_devices
            .iter()
            .find(|device| device.vendor_id == 0x05ac)
            .map(device_id)
    }
}

pub fn device_id(device: &UsbDeviceInfo) -> String {
    format!(
        "{:04x}:{:04x}:{:02x}:{:02x}",
        device.vendor_id,
        device.product_id,
        device.bus_number.unwrap_or_default(),
        device.address.unwrap_or_default()
    )
}

#[derive(Debug, Error)]
pub enum WirelessError {
    #[error("wireless pairing store is corrupt")]
    CorruptStore,
    #[error("wireless store IO failure: {0}")]
    Io(std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{PairingRecord, PairingStore};

    fn temp_store_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        path.push(format!("linux-carplay-pairing-{nanos}.db"));
        path
    }

    #[test]
    fn upsert_and_reload_pairing_store() {
        let path = temp_store_path();
        let mut store = PairingStore::default();
        store.upsert(PairingRecord {
            device_id: "d1".to_string(),
            host_id: "h1".to_string(),
            reconnect_token: "token".to_string(),
            last_seen_unix: 1,
        });
        store.save(&path).expect("save store");

        let reloaded = PairingStore::load(&path).expect("load store");
        assert!(reloaded.find("d1").is_some());

        let _ = std::fs::remove_file(path);
    }
}
