//! Dynamic FFI loader — loads the OverDrive native library at runtime.
//!
//! Used when the SDK is installed via crates.io (without engine source code).
//! The prebuilt binary must be downloaded from GitHub Releases.

use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::OnceLock;

static LIB: OnceLock<Library> = OnceLock::new();

/// Find and load the native library.
fn load_library() -> &'static Library {
    LIB.get_or_init(|| {
        let lib_name = if cfg!(target_os = "windows") {
            "overdrive.dll"
        } else if cfg!(target_os = "macos") {
            "liboverdrive.dylib"
        } else {
            "liboverdrive.so"
        };

        // Search paths
        let search_dirs: Vec<PathBuf> = vec![
            std::env::current_dir().unwrap_or_default(),
            std::env::current_dir().unwrap_or_default().join("lib"),
            std::env::current_exe()
                .unwrap_or_default()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf(),
        ];

        for dir in &search_dirs {
            let path = dir.join(lib_name);
            if path.exists() {
                unsafe {
                    return Library::new(&path).unwrap_or_else(|e| {
                        panic!(
                            "overdrive-sdk: Found {} but failed to load: {}\n\
                             Download from: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases",
                            path.display(), e
                        )
                    });
                }
            }
        }

        // Try system library path
        unsafe {
            Library::new(lib_name).unwrap_or_else(|_| {
                panic!(
                    "overdrive-sdk: Native library '{}' not found!\n\n\
                     Download it from:\n  \
                     https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest\n\n\
                     Place the binary in your project directory or on your system PATH.\n\
                     See: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK#install",
                    lib_name
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

impl Drop for NativeDB {
    fn drop(&mut self) {
        self.close();
    }
}
