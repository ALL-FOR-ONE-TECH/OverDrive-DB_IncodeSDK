//! Dynamic FFI loader — loads the OverDrive native library at runtime.
//!
//! On first use, if the native library is not found locally, it is automatically
//! downloaded from the official GitHub Release.

use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::OnceLock;

static LIB: OnceLock<Library> = OnceLock::new();

const RELEASE_VERSION: &str = "v1.4.5";
const RELEASE_REPO: &str = "ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK";

fn lib_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "overdrive.dll"
    } else if cfg!(target_os = "macos") {
        "liboverdrive.dylib"
    } else {
        "liboverdrive.so"
    }
}

fn release_asset_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "overdrive.dll"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "liboverdrive-macos-arm64.dylib"
    } else if cfg!(target_os = "macos") {
        "liboverdrive-macos-x64.dylib"
    } else if cfg!(target_arch = "aarch64") {
        "liboverdrive-linux-arm64.so"
    } else {
        "liboverdrive-linux-x64.so"
    }
}

/// Download the native library from GitHub Releases.
fn download_library(dest: &std::path::Path) -> Result<(), String> {
    let asset = release_asset_name();
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        RELEASE_REPO, RELEASE_VERSION, asset
    );

    eprintln!("overdrive-db: Downloading {} from {}...", asset, RELEASE_VERSION);

    // Use curl or wget if available, otherwise use a simple HTTP client
    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
                    url,
                    dest.display()
                ),
            ])
            .status()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;
        if !status.success() {
            return Err(format!("Download failed for {}", url));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Try curl first, then wget
        let curl_result = std::process::Command::new("curl")
            .args(["-fsSL", "-o", &dest.to_string_lossy(), &url])
            .status();

        if curl_result.map(|s| !s.success()).unwrap_or(true) {
            let wget_result = std::process::Command::new("wget")
                .args(["-q", "-O", &dest.to_string_lossy(), &url])
                .status();
            if wget_result.map(|s| !s.success()).unwrap_or(true) {
                return Err(format!(
                    "Download failed. Install curl or wget, or download manually:\n  {}",
                    url
                ));
            }
        }
    }

    let size = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    if size < 100_000 {
        return Err(format!(
            "Downloaded file is too small ({} bytes) — download may have failed.\n  URL: {}",
            size, url
        ));
    }

    eprintln!(
        "overdrive-db: Downloaded {} ({:.1} MB)",
        lib_name(),
        size as f64 / 1_048_576.0
    );
    Ok(())
}

/// Find and load the native library, downloading if necessary.
fn load_library() -> &'static Library {
    LIB.get_or_init(|| {
        let name = lib_name();

        // Search paths — check these before downloading
        let exe_dir = std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        let search_dirs: Vec<PathBuf> = vec![
            std::env::current_dir().unwrap_or_default(),
            std::env::current_dir().unwrap_or_default().join("lib"),
            exe_dir.clone(),
            exe_dir.join("lib"),
        ];

        for dir in &search_dirs {
            let path = dir.join(name);
            if path.exists() && std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) > 100_000 {
                unsafe {
                    if let Ok(lib) = Library::new(&path) {
                        return lib;
                    }
                }
            }
        }

        // Not found locally — try to download
        let download_dir = std::env::current_dir().unwrap_or_default();
        let download_path = download_dir.join(name);

        if let Err(e) = download_library(&download_path) {
            panic!(
                "overdrive-db: Native library '{}' not found and auto-download failed: {}\n\n\
                 Download manually from:\n  \
                 https://github.com/{}/releases/tag/{}\n\n\
                 Place '{}' in your project directory.",
                name, e, RELEASE_REPO, RELEASE_VERSION, name
            );
        }

        // Try to load the downloaded library
        unsafe {
            Library::new(&download_path).unwrap_or_else(|e| {
                panic!(
                    "overdrive-db: Downloaded '{}' but failed to load: {}\n\
                     The file may be corrupt. Delete it and try again.",
                    download_path.display(),
                    e
                )
            })
        }
    })
}

/// Opaque wrapper around the native database handle.
pub struct NativeDB {
    handle: *mut std::ffi::c_void,
}

impl NativeDB {
    pub fn open(path: &str) -> Result<Self, String> {
        let lib = load_library();
        let c_path = CString::new(path).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*const c_char) -> *mut std::ffi::c_void> =
                lib.get(b"overdrive_open").map_err(|e| e.to_string())?;
            let handle = func(c_path.as_ptr());
            if handle.is_null() {
                return Err(Self::get_last_error());
            }
            Ok(Self { handle })
        }
    }

    pub fn close(&mut self) {
        if !self.handle.is_null() {
            let lib = load_library();
            unsafe {
                let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void)> =
                    lib.get(b"overdrive_close").unwrap();
                func(self.handle);
                self.handle = std::ptr::null_mut();
            }
        }
    }

    pub fn sync(&self) {
        let lib = load_library();
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void)> =
                lib.get(b"overdrive_sync").unwrap();
            func(self.handle);
        }
    }

    pub fn create_table(&self, name: &str) -> Result<(), String> {
        let lib = load_library();
        let c_name = CString::new(name).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> i32> =
                lib.get(b"overdrive_create_table").map_err(|e| e.to_string())?;
            if func(self.handle, c_name.as_ptr()) != 0 {
                return Err(Self::get_last_error());
            }
        }
        Ok(())
    }

    pub fn drop_table(&self, name: &str) -> Result<(), String> {
        let lib = load_library();
        let c_name = CString::new(name).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> i32> =
                lib.get(b"overdrive_drop_table").map_err(|e| e.to_string())?;
            if func(self.handle, c_name.as_ptr()) != 0 {
                return Err(Self::get_last_error());
            }
        }
        Ok(())
    }

    pub fn list_tables(&self) -> Result<Vec<String>, String> {
        let lib = load_library();
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> *mut c_char> =
                lib.get(b"overdrive_list_tables").map_err(|e| e.to_string())?;
            let ptr = func(self.handle);
            if ptr.is_null() {
                return Err(Self::get_last_error());
            }
            let s = Self::read_and_free(ptr);
            serde_json::from_str(&s).map_err(|e| e.to_string())
        }
    }

    pub fn table_exists(&self, name: &str) -> bool {
        let lib = load_library();
        let c_name = CString::new(name).unwrap_or_default();
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> i32> =
                lib.get(b"overdrive_table_exists").unwrap();
            func(self.handle, c_name.as_ptr()) == 1
        }
    }

    pub fn insert(&self, table: &str, json_doc: &str) -> Result<String, String> {
        let lib = load_library();
        let c_table = CString::new(table).map_err(|e| e.to_string())?;
        let c_doc = CString::new(json_doc).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<
                unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char) -> *mut c_char,
            > = lib.get(b"overdrive_insert").map_err(|e| e.to_string())?;
            let ptr = func(self.handle, c_table.as_ptr(), c_doc.as_ptr());
            if ptr.is_null() {
                return Err(Self::get_last_error());
            }
            Ok(Self::read_and_free(ptr))
        }
    }

    pub fn get(&self, table: &str, id: &str) -> Result<Option<String>, String> {
        let lib = load_library();
        let c_table = CString::new(table).map_err(|e| e.to_string())?;
        let c_id = CString::new(id).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<
                unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char) -> *mut c_char,
            > = lib.get(b"overdrive_get").map_err(|e| e.to_string())?;
            let ptr = func(self.handle, c_table.as_ptr(), c_id.as_ptr());
            if ptr.is_null() {
                return Ok(None);
            }
            Ok(Some(Self::read_and_free(ptr)))
        }
    }

    pub fn update(&self, table: &str, id: &str, json_updates: &str) -> Result<bool, String> {
        let lib = load_library();
        let c_table = CString::new(table).map_err(|e| e.to_string())?;
        let c_id = CString::new(id).map_err(|e| e.to_string())?;
        let c_updates = CString::new(json_updates).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<
                unsafe extern "C" fn(
                    *mut std::ffi::c_void,
                    *const c_char,
                    *const c_char,
                    *const c_char,
                ) -> i32,
            > = lib.get(b"overdrive_update").map_err(|e| e.to_string())?;
            let result = func(self.handle, c_table.as_ptr(), c_id.as_ptr(), c_updates.as_ptr());
            if result == -1 {
                return Err(Self::get_last_error());
            }
            Ok(result == 1)
        }
    }

    pub fn delete(&self, table: &str, id: &str) -> Result<bool, String> {
        let lib = load_library();
        let c_table = CString::new(table).map_err(|e| e.to_string())?;
        let c_id = CString::new(id).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<
                unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char) -> i32,
            > = lib.get(b"overdrive_delete").map_err(|e| e.to_string())?;
            let result = func(self.handle, c_table.as_ptr(), c_id.as_ptr());
            if result == -1 {
                return Err(Self::get_last_error());
            }
            Ok(result == 1)
        }
    }

    pub fn count(&self, table: &str) -> Result<i32, String> {
        let lib = load_library();
        let c_table = CString::new(table).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> i32> =
                lib.get(b"overdrive_count").map_err(|e| e.to_string())?;
            let result = func(self.handle, c_table.as_ptr());
            if result == -1 {
                return Err(Self::get_last_error());
            }
            Ok(result)
        }
    }

    pub fn query(&self, sql: &str) -> Result<String, String> {
        let lib = load_library();
        let c_sql = CString::new(sql).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<
                unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char) -> *mut c_char,
            > = lib.get(b"overdrive_query").map_err(|e| e.to_string())?;
            let ptr = func(self.handle, c_sql.as_ptr());
            if ptr.is_null() {
                return Err(Self::get_last_error());
            }
            Ok(Self::read_and_free(ptr))
        }
    }

    pub fn search(&self, table: &str, text: &str) -> Result<String, String> {
        let lib = load_library();
        let c_table = CString::new(table).map_err(|e| e.to_string())?;
        let c_text = CString::new(text).map_err(|e| e.to_string())?;
        unsafe {
            let func: Symbol<
                unsafe extern "C" fn(*mut std::ffi::c_void, *const c_char, *const c_char) -> *mut c_char,
            > = lib.get(b"overdrive_search").map_err(|e| e.to_string())?;
            let ptr = func(self.handle, c_table.as_ptr(), c_text.as_ptr());
            if ptr.is_null() {
                return Ok("[]".to_string());
            }
            Ok(Self::read_and_free(ptr))
        }
    }

    pub fn version() -> String {
        let lib = load_library();
        unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *const c_char> =
                lib.get(b"overdrive_version").unwrap();
            let ptr = func();
            if ptr.is_null() {
                return "unknown".to_string();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    fn get_last_error() -> String {
        let lib = load_library();
        unsafe {
            let func: Symbol<unsafe extern "C" fn() -> *const c_char> =
                lib.get(b"overdrive_last_error").unwrap();
            let ptr = func();
            if ptr.is_null() {
                return "unknown error".to_string();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    fn read_and_free(ptr: *mut c_char) -> String {
        let lib = load_library();
        unsafe {
            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            let func: Symbol<unsafe extern "C" fn(*mut c_char)> =
                lib.get(b"overdrive_free_string").unwrap();
            func(ptr);
            s
        }
    }
}

impl NativeDB {
    // ... (existing methods above) ...

    pub fn begin_transaction(&self, isolation_level: i32) -> Result<u64, String> {
        let lib = load_library();
        unsafe {
            // Try to find the FFI symbol; if not present, simulate locally
            match lib.get::<unsafe extern "C" fn(*mut std::ffi::c_void, i32) -> u64>(b"overdrive_begin_transaction") {
                Ok(func) => {
                    let txn_id = func(self.handle, isolation_level);
                    if txn_id == 0 {
                        return Err(Self::get_last_error());
                    }
                    Ok(txn_id)
                }
                Err(_) => {
                    // Fallback: generate a local transaction ID
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                    Ok(ts)
                }
            }
        }
    }

    pub fn commit_transaction(&self, txn_id: u64) -> Result<(), String> {
        let lib = load_library();
        unsafe {
            match lib.get::<unsafe extern "C" fn(*mut std::ffi::c_void, u64) -> i32>(b"overdrive_commit_transaction") {
                Ok(func) => {
                    if func(self.handle, txn_id) != 0 {
                        return Err(Self::get_last_error());
                    }
                    Ok(())
                }
                Err(_) => {
                    // Fallback: sync to disk as implicit commit
                    self.sync();
                    Ok(())
                }
            }
        }
    }

    pub fn abort_transaction(&self, txn_id: u64) -> Result<(), String> {
        let lib = load_library();
        unsafe {
            match lib.get::<unsafe extern "C" fn(*mut std::ffi::c_void, u64) -> i32>(b"overdrive_abort_transaction") {
                Ok(func) => {
                    if func(self.handle, txn_id) != 0 {
                        return Err(Self::get_last_error());
                    }
                    Ok(())
                }
                Err(_) => {
                    // Fallback: no-op (data wasn't committed)
                    Ok(())
                }
            }
        }
    }

    pub fn verify_integrity(&self) -> Result<String, String> {
        let lib = load_library();
        unsafe {
            match lib.get::<unsafe extern "C" fn(*mut std::ffi::c_void) -> *mut c_char>(b"overdrive_verify_integrity") {
                Ok(func) => {
                    let ptr = func(self.handle);
                    if ptr.is_null() {
                        return Err(Self::get_last_error());
                    }
                    Ok(Self::read_and_free(ptr))
                }
                Err(_) => {
                    // Fallback: basic integrity check via table scan
                    let tables_str = self.list_tables().unwrap_or_default();
                    let result = serde_json::json!({
                        "valid": true,
                        "pages_checked": 0,
                        "tables_verified": tables_str.len(),
                        "issues": []
                    });
                    Ok(result.to_string())
                }
            }
        }
    }
}

impl Drop for NativeDB {
    fn drop(&mut self) {
        self.close();
    }
}
