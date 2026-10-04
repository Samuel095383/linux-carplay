use thiserror::Error;

#[cfg(feature = "libusb")]
use rusb::UsbContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub bus_number: Option<u8>,
    pub address: Option<u8>,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("optional USB backend is unavailable in this build")]
    BackendUnavailable,
    #[error("USB transport error: {0}")]
    Generic(String),
}

pub trait UsbDiscoveryBackend {
    fn discover(&self) -> Result<Vec<UsbDeviceInfo>, TransportError>;
}

pub struct MockUsbDiscovery;

impl UsbDiscoveryBackend for MockUsbDiscovery {
    fn discover(&self) -> Result<Vec<UsbDeviceInfo>, TransportError> {
        Ok(vec![UsbDeviceInfo {
            vendor_id: 0x05ac,
            product_id: 0x12a8,
            bus_number: Some(1),
            address: Some(1),
        }])
    }
}

pub struct NullUsbDiscovery;

impl UsbDiscoveryBackend for NullUsbDiscovery {
    fn discover(&self) -> Result<Vec<UsbDeviceInfo>, TransportError> {
        Err(TransportError::BackendUnavailable)
    }
}

#[cfg(feature = "libusb")]
pub struct LibusbDiscovery;

#[cfg(feature = "libusb")]
impl UsbDiscoveryBackend for LibusbDiscovery {
    fn discover(&self) -> Result<Vec<UsbDeviceInfo>, TransportError> {
        let context =
            rusb::Context::new().map_err(|err| TransportError::Generic(err.to_string()))?;
        let devices = context
            .devices()
            .map_err(|err| TransportError::Generic(err.to_string()))?;

        let mut discovered = Vec::new();
        for device in devices.iter() {
            let descriptor = device
                .device_descriptor()
                .map_err(|err| TransportError::Generic(err.to_string()))?;
            discovered.push(UsbDeviceInfo {
                vendor_id: descriptor.vendor_id(),
                product_id: descriptor.product_id(),
                bus_number: Some(device.bus_number()),
                address: Some(device.address()),
            });
        }

        Ok(discovered)
    }
}

#[cfg(not(feature = "libusb"))]
pub struct LibusbDiscovery;

#[cfg(not(feature = "libusb"))]
impl UsbDiscoveryBackend for LibusbDiscovery {
    fn discover(&self) -> Result<Vec<UsbDeviceInfo>, TransportError> {
        Err(TransportError::BackendUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::{MockUsbDiscovery, NullUsbDiscovery, TransportError, UsbDiscoveryBackend};

    #[test]
    fn mock_discovery_returns_device() {
        let devices = MockUsbDiscovery
            .discover()
            .expect("mock discovery should succeed");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].vendor_id, 0x05ac);
    }

    #[test]
    fn null_discovery_returns_backend_unavailable() {
        let err = NullUsbDiscovery
            .discover()
            .expect_err("null backend should fail");
        assert!(matches!(err, TransportError::BackendUnavailable));
    }
}
