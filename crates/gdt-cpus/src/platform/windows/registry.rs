//! Windows-specific CPU information detection via the system registry.
//!
//! This module provides functionality to query the Windows Registry for CPU details,
//! such as the processor name string, vendor identifier, and other hardware identifiers.
//! This is typically used as a fallback mechanism if other detection methods
//! (like `cpuid` or `GetLogicalProcessorInformationEx`) are insufficient or unavailable.
//!
//! The main entry point is [`detect_via_registry`], which attempts to read values from
//! `HKEY_LOCAL_MACHINE\\HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0`.

use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ, REG_VALUE_TYPE, RegCloseKey, RegOpenKeyExW,
    RegQueryValueExW,
};
use windows::core::{Error as WinError, HRESULT, PCWSTR, Result as WinResult, w};

use crate::{Error, Result, Vendor};

use super::utils::to_wide_null_vec;

/// RAII helper structure for managing a Windows Registry key (`HKEY`).
///
/// This guard ensures that an opened registry key is automatically closed via `RegCloseKey`
/// when the guard instance goes out of scope, preventing resource leaks.
///
/// # Fields
///
/// * `0`: The `HKEY` handle to the opened registry key.
struct RegistryKeyGuard(HKEY);

impl RegistryKeyGuard {
    /// Creates a new `RegistryKeyGuard` for the given `HKEY`.
    ///
    /// It is assumed that `hkey` is a valid, opened key handle. The guard
    /// will attempt to close this handle upon dropping.
    fn new(hkey: HKEY) -> Self {
        Self(hkey)
    }
}

impl Drop for RegistryKeyGuard {
    fn drop(&mut self) {
        // HKEY(0) or HKEY(-1) are considered invalid and should not be closed.
        // Valid HKEYs returned by RegOpenKeyExW are not 0 or -1.
        if !self.0.is_invalid() {
            // SAFETY: self.0 is a valid HKEY handle that was successfully opened.
            // RegCloseKey is the standard way to release it.
            // Ignoring the return value of RegCloseKey is common as there's
            // little that can be done about a failure to close a key.
            unsafe {
                let _ = RegCloseKey(self.0);
            };
        }
    }
}

/// Reads a string value (REG_SZ) from an opened registry key.
///
/// This function queries for the size and type of the specified registry value,
/// then reads the string data if it's of type `REG_SZ`.
///
/// # Arguments
///
/// * `hkey`: An `HKEY` handle to an opened registry key.
/// * `value_name`: The name of the registry value to read (e.g., "ProcessorNameString").
///
/// # Returns
///
/// A `WinResult<String>` containing the string value if successful, or a `WinError`
/// if the value cannot be read, is not a `REG_SZ` type, or if UTF-16 conversion fails.
fn read_registry_string_value(hkey: HKEY, value_name: &str) -> WinResult<String> {
    let wide_value_name_vec = to_wide_null_vec(value_name);
    let pcwstr_value_name = PCWSTR(wide_value_name_vec.as_ptr());

    let mut data_type: REG_VALUE_TYPE = REG_VALUE_TYPE(0);
    let mut buffer_size_bytes: u32 = 0;

    // Step 1: Query the size of the data (in bytes) and the value type.
    // SAFETY: Calling Windows API. Arguments are correctly formulated.
    // `pcwstr_value_name` points to valid memory (from `wide_value_name_vec`).
    let mut win_err_code = unsafe {
        RegQueryValueExW(
            hkey,
            pcwstr_value_name,
            None, // reserved, must be None
            Some(&mut data_type),
            None,                         // data buffer (lpData = null to get size)
            Some(&mut buffer_size_bytes), // data buffer size in bytes
        )
    };

    if win_err_code != ERROR_SUCCESS {
        return Err(WinError::from_hresult(HRESULT::from_win32(win_err_code.0)));
    }

    // Ensure the type is REG_SZ (string).
    if data_type != REG_SZ {
        return Err(WinError::new(
            windows::Win32::Foundation::E_UNEXPECTED,
            "Registry value is not of type REG_SZ",
        ));
    }

    if buffer_size_bytes == 0 {
        // Value is an empty string (contains only null terminator, 0 data bytes).
        return Ok(String::new());
    }

    // Buffer size must be even for UTF-16 data.
    if buffer_size_bytes % 2 != 0 {
        return Err(WinError::new(
            windows::Win32::Foundation::E_INVALIDARG,
            "Registry value (REG_SZ) size in bytes is not an even number.",
        ));
    }

    // Prepare a buffer for the data (number of u16 elements).
    // `buffer_size_bytes` includes the size of the data AND the null terminator.
    let num_u16_elements = (buffer_size_bytes / 2) as usize;
    let mut buffer_u16: Vec<u16> = vec![0u16; num_u16_elements];

    // Step 2: Retrieve the actual data.
    // SAFETY: Calling Windows API. `buffer_u16.as_mut_ptr()` is a valid pointer,
    // `buffer_size_bytes` (original value from the first call) is the size of the allocated buffer.
    // After the call, `buffer_size_bytes` will be updated to the actual size written.
    let mut actual_bytes_written = buffer_size_bytes; // Pass the allocated buffer size
    win_err_code = unsafe {
        RegQueryValueExW(
            hkey,
            pcwstr_value_name,
            None,                                     // reserved
            None,                                     // data type (already have it, or pass None)
            Some(buffer_u16.as_mut_ptr() as *mut u8), // pointer to the data buffer
            Some(&mut actual_bytes_written), // input: buffer size, output: actual bytes written
        )
    };

    if win_err_code != ERROR_SUCCESS {
        return Err(WinError::from_hresult(HRESULT::from_win32(win_err_code.0)));
    }

    // `actual_bytes_written` now contains the actual number of bytes written, including the null terminator.
    // Convert to the number of u16 elements.
    let actual_u16s_written = (actual_bytes_written / 2) as usize;

    // Create a slice from the actually written data.
    // `String::from_utf16` will stop at the first null character and not include it in the String.
    let relevant_slice = &buffer_u16[..actual_u16s_written.min(buffer_u16.len())];

    String::from_utf16(relevant_slice).map_err(|e| {
        WinError::new(
            windows::Win32::Foundation::E_UNEXPECTED,
            format!("Failed to convert registry value from UTF-16: {}", e),
        )
    })
}

/// Attempts to detect CPU vendor and model name by querying the Windows Registry.
///
/// This function serves as a fallback mechanism. It queries the registry key
/// `HKEY_LOCAL_MACHINE\\HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0` for
/// values like "ProcessorNameString", "Identifier", and "VendorIdentifier"
/// to determine or refine the CPU's model name and vendor.
///
/// # Arguments
///
/// * `vendor`: A mutable reference to a [`Vendor`] enum, which will be updated if
///   the vendor can be determined from the registry.
/// * `model_name`: A mutable reference to a `String`, which will be updated with
///   the "ProcessorNameString" value if found.
///
/// # Errors
///
/// Returns `Error::Detection` if the primary registry key
/// (`System\\CentralProcessor\\0`) cannot be opened. Failures to read individual
/// values within the key are logged as debug messages but do not return an error,
/// allowing the function to proceed with any information it can gather.
///
/// # Remarks
///
/// This function updates `vendor` and `model_name` in place.
/// It prioritizes information already present; for example, it only attempts to
/// determine the vendor from "Identifier" or "VendorIdentifier" if `*vendor`
/// is still `Vendor::Unknown`.
pub(crate) fn detect_via_registry(vendor: &mut Vendor, model_name: &mut String) -> Result<()> {
    // crate::Result
    let pcwstr_subkey: PCWSTR = w!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0");
    let mut hkey_opened = HKEY::default();

    // SAFETY: Calling Windows API to open a registry key.
    // `pcwstr_subkey` is a valid pointer to a constant string.
    let win_err_code_open = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            pcwstr_subkey,
            Some(0),  // ulOptions, must be 0
            KEY_READ, // u32 value of the KEY_READ flag
            &mut hkey_opened,
        )
    };

    if win_err_code_open != ERROR_SUCCESS {
        return Err(Error::Detection(format!(
            "Failed to open registry key '{}': {}",
            r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            WinError::from_hresult(HRESULT::from_win32(win_err_code_open.0))
        )));
    }

    // Create an RAII guard for the opened key to ensure it's closed.
    let hkey_guard = RegistryKeyGuard::new(hkey_opened);
    let hkey = hkey_guard.0; // Use the HKEY from the guard for subsequent operations

    // 1) Read ProcessorNameString
    match read_registry_string_value(hkey, "ProcessorNameString") {
        Ok(name_str) => {
            log::debug!("Registry Fallback: ProcessorNameString = '{}'", name_str);
            *model_name = name_str;
        }
        Err(e) => {
            // For a fallback mechanism, failure to read a single value might not be critical.
            // Log it and continue.
            log::debug!(
                "Registry Fallback: Failed to read ProcessorNameString: {}",
                e
            );
        }
    }

    // 2) If "apple" is in the model name (already read) → set Vendor to Apple
    if model_name.to_lowercase().contains("apple") {
        *vendor = Vendor::Apple;
        log::debug!(
            "Registry Fallback: Vendor set to Apple based on model name: '{}'",
            model_name
        );
    }

    // 3) Read Identifier → if ARMv8/ARM64 → set Vendor to Arm (mainly for Windows on ARM)
    if *vendor == Vendor::Unknown {
        // Only check if vendor is not yet determined
        match read_registry_string_value(hkey, "Identifier") {
            Ok(idf_str) => {
                log::debug!("Registry Fallback: Identifier = '{}'", idf_str);
                let lower_idf = idf_str.to_lowercase();
                // Windows on ARM often has "ARMv8" or similar in Identifier
                if lower_idf.contains("armv8") || lower_idf.contains("arm64") {
                    *vendor = Vendor::Arm;
                    log::debug!(
                        "Registry Fallback: Vendor set to ARM based on Identifier: '{}'",
                        idf_str
                    );
                }
            }
            Err(e) => {
                log::debug!("Registry Fallback: Failed to read Identifier: {}", e);
            }
        }
    }

    // 4) Read VendorIdentifier
    if *vendor == Vendor::Unknown {
        // Only check if vendor is not yet determined
        match read_registry_string_value(hkey, "VendorIdentifier") {
            Ok(vid_str) => {
                log::debug!("Registry Fallback: VendorIdentifier = '{}'", vid_str);
                let lower_vid = vid_str.trim().to_lowercase();
                match lower_vid.as_str() {
                    "genuineintel" => *vendor = Vendor::Intel,
                    "authenticamd" => *vendor = Vendor::Amd,
                    "apple" => *vendor = Vendor::Apple, // Should already be caught, but for robustness
                    other_vid if !other_vid.is_empty() => {
                        *vendor = Vendor::Other(other_vid.to_string());
                        log::debug!(
                            "Registry Fallback: Vendor set to Other based on VendorIdentifier: '{}'",
                            other_vid
                        );
                    }
                    _ => {} // Empty or unhandled
                }
            }
            Err(e) => {
                log::debug!("Registry Fallback: Failed to read VendorIdentifier: {}", e);
            }
        }
    }

    // The `hkey` will be automatically closed by the `Drop` implementation of `hkey_guard`
    // when `hkey_guard` goes out of scope.
    Ok(())
}
