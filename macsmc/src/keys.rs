//! SMC key definitions for Apple Silicon M2 and other macOS systems

/// SMC key structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmcKey {
    /// 4-character SMC key
    pub key: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// Key category
    pub category: KeyCategory,
    /// Expected data type
    pub data_type: KeyDataType,
}

/// SMC key categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCategory {
    /// CPU related sensors
    Cpu,
    /// GPU related sensors  
    Gpu,
    /// System sensors (memory, storage, etc.)
    System,
    /// General sensors (airflow, ambient, etc.)
    Sensor,
    /// Fan control
    Fan,
    /// Battery
    Battery,
    /// Power management
    Power,
}

/// Expected data types for SMC keys
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyDataType {
    /// Temperature values
    Temperature,
    /// Voltage values
    Voltage,
    /// Current values
    Current,
    /// Power values
    Power,
    /// Fan speed (RPM)
    FanSpeed,
    /// Boolean flags
    Flag,
    /// Generic numeric
    Numeric,
    /// String data
    String,
}

impl SmcKey {
    /// Create a new SMC key
    pub const fn new(
        key: &'static str,
        name: &'static str,
        category: KeyCategory,
        data_type: KeyDataType,
    ) -> Self {
        Self {
            key,
            name,
            category,
            data_type,
        }
    }
}

/// Convert 4-character string to u32 key
pub fn key_to_u32(key: &str) -> u32 {
    if key.len() != 4 {
        return 0;
    }
    let bytes = key.as_bytes();
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Convert u32 key back to string
pub fn u32_to_key(key: u32) -> String {
    let bytes = key.to_be_bytes();
    String::from_utf8_lossy(&bytes).to_string()
}
