//! SMC client implementation

use crate::{data::SmcData, error::*, keys::key_to_u32};
use std::{mem::size_of, os::raw::c_void};

/// SMC client for raw key reading
#[derive(Debug)]
pub struct SmcClient {
    connection: SMCConnection,
}

impl SmcClient {
    /// Create a new SMC client connection
    pub fn new() -> Result<Self> {
        let connection = SMCConnection::new()?;
        Ok(Self { connection })
    }

    /// Read a raw SMC key and return parsed data
    pub fn read_key(&mut self, key: &str) -> Result<SmcData> {
        if key.len() != 4 {
            return Err(SmcError::InvalidKey(key.to_string()));
        }

        let key_u32 = key_to_u32(key);
        let result = self.connection.read_key(key_u32)?;
        let data_type = u32_to_string(result.data_type);
        
        SmcData::parse(&result.data[..result.data_size as usize], &data_type)
            .map_err(|mut e| {
                if let SmcError::DataError { key: ref mut k, .. } = e {
                    *k = key.to_string();
                }
                e
            })
    }

    /// Get key information without reading data
    pub fn get_key_info(&mut self, key: &str) -> Result<SmcKeyInfo> {
        if key.len() != 4 {
            return Err(SmcError::InvalidKey(key.to_string()));
        }

        let key_u32 = key_to_u32(key);
        let info = self.connection.get_key_info(key_u32)?;
        
        Ok(SmcKeyInfo {
            key: key.to_string(),
            data_type: u32_to_string(info.data_type),
            data_size: info.data_size,
        })
    }

    /// Get the total number of SMC keys
    pub fn get_key_count(&mut self) -> Result<u32> {
        let data = self.read_key("#KEY")?;
        data.as_u64().map(|v| v as u32)
    }

    /// List all available SMC keys (warning: slow operation)
    pub fn list_all_keys(&mut self) -> Result<Vec<String>> {
        let count = self.get_key_count()?;
        let mut keys = Vec::new();

        for i in 0..count {
            if let Ok(info) = self.connection.get_key_by_index(i) {
                keys.push(u32_to_string(info.key));
            }
        }

        Ok(keys)
    }
}

/// SMC key information
#[derive(Debug, Clone)]
pub struct SmcKeyInfo {
    /// The SMC key
    pub key: String,
    /// Data type string
    pub data_type: String,
    /// Size of data in bytes
    pub data_size: u32,
}

fn u32_to_string(value: u32) -> String {
    let bytes = value.to_be_bytes();
    String::from_utf8_lossy(&bytes).to_string()
}

// Low-level SMC interface (FFI)

#[derive(Debug)]
struct SMCConnection {
    connection: IoConnectT,
}

impl Drop for SMCConnection {
    fn drop(&mut self) {
        unsafe {
            IOServiceClose(self.connection);
        }
    }
}

impl SMCConnection {
    fn new() -> Result<Self> {
        let connection = unsafe { smc_open()? };
        Ok(Self { connection })
    }

    fn read_key(&mut self, key: u32) -> Result<SmcResult> {
        unsafe { smc_read_key(self.connection, key) }
    }

    fn get_key_info(&mut self, key: u32) -> Result<SmcKeyInfo2> {
        unsafe { smc_get_key_info(self.connection, key) }
    }

    fn get_key_by_index(&mut self, index: u32) -> Result<SmcKeyInfo2> {
        unsafe { smc_get_key_by_index(self.connection, index) }
    }
}

#[derive(Debug)]
struct SmcResult {
    data_type: u32,
    data_size: u32,
    data: [u8; 32],
}

#[derive(Debug)]
struct SmcKeyInfo2 {
    key: u32,
    data_type: u32,
    data_size: u32,
}

// FFI types and functions
type KernReturnT = i32;
type IoConnectT = *mut c_void;
type IoServiceT = *mut c_void;
type MachPortT = *mut c_void;

const KERN_SUCCESS: KernReturnT = 0;
const RETURN_NOT_PRIVILEGED: KernReturnT = 0x10000000 | 0x2c1;

#[repr(C)]
struct SMCKeyData {
    key: u32,
    version: SMCKeyDataVersion,
    p_limit_data: SMCKeyDataLimitData,
    key_info: SMCKeyDataKeyInfo,
    result: u8,
    status: u8,
    data8: u8,
    data32: u32,
    bytes: [u8; 32],
}

#[repr(C)]
struct SMCKeyDataVersion {
    major: u8,
    minor: u8,
    build: u8,
    reserved: u8,
    release: u16,
}

#[repr(C)]
struct SMCKeyDataLimitData {
    version: u16,
    length: u16,
    cpu_plimit: u32,
    gpu_plimit: u32,
    mem_plimit: u32,
}

#[repr(C)]
struct SMCKeyDataKeyInfo {
    data_size: u32,
    data_type: u32,
    data_attributes: u8,
}

impl Default for SMCKeyData {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for SMCKeyDataVersion {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for SMCKeyDataLimitData {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for SMCKeyDataKeyInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOServiceMatching(name: *const u8) -> *mut c_void;
    fn IOServiceGetMatchingService(masterPort: MachPortT, matching: *mut c_void) -> IoServiceT;
    fn IOServiceOpen(
        service: IoServiceT,
        owningTask: MachPortT,
        type_: u32,
        connect: *mut IoConnectT,
    ) -> KernReturnT;
    fn IOServiceClose(connect: IoConnectT) -> KernReturnT;
    fn IOConnectCallStructMethod(
        connection: IoConnectT,
        selector: u32,
        input: *const c_void,
        inputSize: usize,
        output: *mut c_void,
        outputSize: *mut usize,
    ) -> KernReturnT;
    fn IOObjectRelease(object: IoServiceT) -> KernReturnT;
    fn mach_task_self() -> MachPortT;
}

unsafe fn smc_open() -> Result<IoConnectT> {
    let matching_dict = IOServiceMatching(b"AppleSMC\0".as_ptr());
    let service = IOServiceGetMatchingService(std::ptr::null_mut(), matching_dict);

    if service.is_null() {
        return Err(SmcError::NotAvailable);
    }

    let mut connection: IoConnectT = std::ptr::null_mut();
    let result = IOServiceOpen(service, mach_task_self(), 0, &mut connection);
    IOObjectRelease(service);

    if result != KERN_SUCCESS {
        return Err(SmcError::SmcError(result));
    }

    Ok(connection)
}

unsafe fn smc_read_key(connection: IoConnectT, key: u32) -> Result<SmcResult> {
    // First get key info
    let mut input = SMCKeyData::default();
    input.key = key;
    input.data8 = 9; // SMC_CMD_READ_KEYINFO

    let mut output = SMCKeyData::default();
    smc_call(connection, &input, &mut output)?;

    let data_type = output.key_info.data_type;
    let data_size = output.key_info.data_size;

    if data_size > 32 {
        return Err(SmcError::DataError {
            key: crate::keys::u32_to_key(key),
            data_type: u32_to_string(data_type),
        });
    }

    // Now read the actual data
    input.key_info.data_size = data_size;
    input.data8 = 5; // SMC_CMD_READ_BYTES

    smc_call(connection, &input, &mut output)?;

    Ok(SmcResult {
        data_type,
        data_size,
        data: output.bytes,
    })
}

unsafe fn smc_get_key_info(connection: IoConnectT, key: u32) -> Result<SmcKeyInfo2> {
    let mut input = SMCKeyData::default();
    input.key = key;
    input.data8 = 9; // SMC_CMD_READ_KEYINFO

    let mut output = SMCKeyData::default();
    smc_call(connection, &input, &mut output)?;

    Ok(SmcKeyInfo2 {
        key,
        data_type: output.key_info.data_type,
        data_size: output.key_info.data_size,
    })
}

unsafe fn smc_get_key_by_index(connection: IoConnectT, index: u32) -> Result<SmcKeyInfo2> {
    let mut input = SMCKeyData::default();
    input.data8 = 8; // SMC_CMD_READ_INDEX
    input.data32 = index;

    let mut output = SMCKeyData::default();
    smc_call(connection, &input, &mut output)?;

    Ok(SmcKeyInfo2 {
        key: output.key,
        data_type: output.key_info.data_type,
        data_size: output.key_info.data_size,
    })
}

unsafe fn smc_call(
    connection: IoConnectT,
    input: &SMCKeyData,
    output: &mut SMCKeyData,
) -> Result<()> {
    let mut output_size = size_of::<SMCKeyData>();

    let result = IOConnectCallStructMethod(
        connection,
        2, // KERNEL_INDEX_SMC
        input as *const _ as *const c_void,
        size_of::<SMCKeyData>(),
        output as *mut _ as *mut c_void,
        &mut output_size,
    );

    match result {
        KERN_SUCCESS => {
            if output.result == 132 {
                // Key not found
                Err(SmcError::InvalidKey("key not found".to_string()))
            } else {
                Ok(())
            }
        }
        RETURN_NOT_PRIVILEGED => Err(SmcError::InsufficientPrivileges),
        _ => Err(SmcError::SmcError(result)),
    }
}