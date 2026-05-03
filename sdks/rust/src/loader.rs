//! Native library loader for OverDrive-DB

use crate::errors::{OverDriveError, Result};
use libloading::{Library, Symbol};
use std::ffi::{c_char, c_int, c_void};
use std::sync::Once;

static INIT: Once = Once::new();
static mut LOADER: Option<NativeLoader> = None;

/// Native library function signatures
type OverdriveOpenFn = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type OverdriveOpenWithPasswordFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void;
type OverdriveOpenWithEngineFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void;
type OverdriveCloseFn = unsafe extern "C" fn(*mut c_void);
type OverdriveCreateTableFn = unsafe extern "C" fn(*mut c_void, *const c_char);
type OverdriveInsertFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> *mut c_char;
type OverdriveGetFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> *mut c_char;
type OverdriveUpdateFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, *const c_char);
type OverdriveDeleteFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char);
type OverdriveQueryFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_char;
type OverdriveQuerySafeFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> *mut c_char;
type OverdriveSearchFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> *mut c_char;
type OverdriveCountFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type OverdriveFreeStringFn = unsafe extern "C" fn(*mut c_char);
type OverdriveVersionFn = unsafe extern "C" fn() -> *const c_char;

/// Native library loader
pub struct NativeLoader {
    _library: Library,
    
    // Function pointers
    pub overdrive_open: OverdriveOpenFn,
    pub overdrive_open_with_password: OverdriveOpenWithPasswordFn,
    pub overdrive_open_with_engine: OverdriveOpenWithEngineFn,
    pub overdrive_close: OverdriveCloseFn,
    pub overdrive_create_table: OverdriveCreateTableFn,
    pub overdrive_insert: OverdriveInsertFn,
    pub overdrive_get: OverdriveGetFn,
    pub overdrive_update: OverdriveUpdateFn,
    pub overdrive_delete: OverdriveDeleteFn,
    pub overdrive_query: OverdriveQueryFn,
    pub overdrive_query_safe: OverdriveQuerySafeFn,
    pub overdrive_search: OverdriveSearchFn,
    pub overdrive_count: OverdriveCountFn,
    pub overdrive_free_string: OverdriveFreeStringFn,
    pub overdrive_version: OverdriveVersionFn,
}

impl NativeLoader {
    /// Create new native library loader
    pub fn new() -> Result<Self> {
        INIT.call_once(|| {
            match Self::load_library() {
                Ok(loader) => unsafe { LOADER = Some(loader) },
                Err(_) => {} // Error will be handled below
            }
        });
        
        unsafe {
            LOADER.as_ref()
                .ok_or_else(|| OverDriveError::LibraryError("Failed to load native library".into()))
                .map(|loader| Self {
                    _library: Library::new(Self::get_library_path()).unwrap(),
                    overdrive_open: loader.overdrive_open,
                    overdrive_open_with_password: loader.overdrive_open_with_password,
                    overdrive_open_with_engine: loader.overdrive_open_with_engine,
                    overdrive_close: loader.overdrive_close,
                    overdrive_create_table: loader.overdrive_create_table,
                    overdrive_insert: loader.overdrive_insert,
                    overdrive_get: loader.overdrive_get,
                    overdrive_update: loader.overdrive_update,
                    overdrive_delete: loader.overdrive_delete,
                    overdrive_query: loader.overdrive_query,
                    overdrive_query_safe: loader.overdrive_query_safe,
                    overdrive_search: loader.overdrive_search,
                    overdrive_count: loader.overdrive_count,
                    overdrive_free_string: loader.overdrive_free_string,
                    overdrive_version: loader.overdrive_version,
                })
        }
    }
    
    /// Load the native library
    fn load_library() -> Result<Self> {
        let library_path = Self::get_library_path();
        let library = Library::new(&library_path)
            .map_err(|e| OverDriveError::LibraryError(format!("Failed to load {}: {}", library_path, e)))?;
        
        // Load function symbols
        let overdrive_open: Symbol<OverdriveOpenFn> = unsafe {
            library.get(b"overdrive_open")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_open not found: {}", e)))?
        };
        
        let overdrive_open_with_password: Symbol<OverdriveOpenWithPasswordFn> = unsafe {
            library.get(b"overdrive_open_with_password")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_open_with_password not found: {}", e)))?
        };
        
        let overdrive_open_with_engine: Symbol<OverdriveOpenWithEngineFn> = unsafe {
            library.get(b"overdrive_open_with_engine")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_open_with_engine not found: {}", e)))?
        };
        
        let overdrive_close: Symbol<OverdriveCloseFn> = unsafe {
            library.get(b"overdrive_close")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_close not found: {}", e)))?
        };
        
        let overdrive_create_table: Symbol<OverdriveCreateTableFn> = unsafe {
            library.get(b"overdrive_create_table")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_create_table not found: {}", e)))?
        };
        
        let overdrive_insert: Symbol<OverdriveInsertFn> = unsafe {
            library.get(b"overdrive_insert")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_insert not found: {}", e)))?
        };
        
        let overdrive_get: Symbol<OverdriveGetFn> = unsafe {
            library.get(b"overdrive_get")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_get not found: {}", e)))?
        };
        
        let overdrive_update: Symbol<OverdriveUpdateFn> = unsafe {
            library.get(b"overdrive_update")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_update not found: {}", e)))?
        };
        
        let overdrive_delete: Symbol<OverdriveDeleteFn> = unsafe {
            library.get(b"overdrive_delete")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_delete not found: {}", e)))?
        };
        
        let overdrive_query: Symbol<OverdriveQueryFn> = unsafe {
            library.get(b"overdrive_query")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_query not found: {}", e)))?
        };
        
        let overdrive_query_safe: Symbol<OverdriveQuerySafeFn> = unsafe {
            library.get(b"overdrive_query_safe")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_query_safe not found: {}", e)))?
        };
        
        let overdrive_search: Symbol<OverdriveSearchFn> = unsafe {
            library.get(b"overdrive_search")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_search not found: {}", e)))?
        };
        
        let overdrive_count: Symbol<OverdriveCountFn> = unsafe {
            library.get(b"overdrive_count")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_count not found: {}", e)))?
        };
        
        let overdrive_free_string: Symbol<OverdriveFreeStringFn> = unsafe {
            library.get(b"overdrive_free_string")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_free_string not found: {}", e)))?
        };
        
        let overdrive_version: Symbol<OverdriveVersionFn> = unsafe {
            library.get(b"overdrive_version")
                .map_err(|e| OverDriveError::LibraryError(format!("Symbol overdrive_version not found: {}", e)))?
        };
        
        Ok(Self {
            _library: library,
            overdrive_open: *overdrive_open,
            overdrive_open_with_password: *overdrive_open_with_password,
            overdrive_open_with_engine: *overdrive_open_with_engine,
            overdrive_close: *overdrive_close,
            overdrive_create_table: *overdrive_create_table,
            overdrive_insert: *overdrive_insert,
            overdrive_get: *overdrive_get,
            overdrive_update: *overdrive_update,
            overdrive_delete: *overdrive_delete,
            overdrive_query: *overdrive_query,
            overdrive_query_safe: *overdrive_query_safe,
            overdrive_search: *overdrive_search,
            overdrive_count: *overdrive_count,
            overdrive_free_string: *overdrive_free_string,
            overdrive_version: *overdrive_version,
        })
    }
    
    /// Get platform-specific library path
    fn get_library_path() -> String {
        // Try to find library in native/ directory first
        let base_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        
        #[cfg(target_os = "windows")]
        let library_name = "overdrive.dll";
        
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let library_name = "liboverdrive.so";
        
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        let library_name = "liboverdrive-arm64.so";
        
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let library_name = "liboverdrive.dylib";
        
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let library_name = "liboverdrive-arm64.dylib";
        
        // Try different locations
        let candidates = vec![
            // Native directory (new structure)
            #[cfg(target_os = "windows")]
            base_path.join("native/windows").join(library_name),
            
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            base_path.join("native/linux/x64").join(library_name),
            
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            base_path.join("native/linux/arm64").join(library_name),
            
            #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
            base_path.join("native/macos/x64").join(library_name),
            
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            base_path.join("native/macos/arm64").join(library_name),
            
            // Current directory
            base_path.join(library_name),
            
            // System library path
            std::path::PathBuf::from(library_name),
        ];
        
        for candidate in candidates {
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
        }
        
        // Fallback to library name (system will search PATH)
        library_name.to_string()
    }
}

// Thread safety
unsafe impl Send for NativeLoader {}
unsafe impl Sync for NativeLoader {}