use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use libloading::{Library, Symbol};
use std::sync::OnceLock;

use crate::error::SdkError;

// BEFORE: `static LIB: OnceLock<Library>` + `load()` that PANICS via
// `panic!(...)` if no candidate path opens the library. Any caller
// (including a single misconfigured OVERDRIVE_LIB_PATH, or simply running
// on a platform/arch that has no bundled .so/.dylib/.dll yet) crashed the
// whole host process — there was no way for an application to catch this
// and degrade gracefully.
//
// AFTER: the same lazy/cached loading, but failures are captured as an
// `Err` inside the OnceLock instead of a panic, and every call site gets a
// `Result` back.
static LIB: OnceLock<Result<Library, String>> = OnceLock::new();

/// Load the native overdrive library (cached after first call).
/// Returns an error instead of panicking if no candidate path succeeds.
pub(crate) fn try_load() -> Result<&'static Library, SdkError> {
    let result = LIB.get_or_init(|| {
        let candidates = lib_candidates();
        for path in &candidates {
            if let Ok(lib) = unsafe { Library::new(path) } {
                return Ok(lib);
            }
        }
        Err(format!(
            "[overdrive-sdk] Could not load native library.\n\
             Tried: {}\n\
             Set OVERDRIVE_LIB_PATH to point to overdrive.dll/liboverdrive.so/liboverdrive.dylib",
            candidates.join(", ")
        ))
    });

    match result {
        Ok(lib) => Ok(lib),
        Err(msg) => Err(SdkError(msg.clone())),
    }
}

fn lib_candidates() -> Vec<String> {
    let mut v = Vec::new();

    // 1. Env override
    if let Ok(p) = std::env::var("OVERDRIVE_LIB_PATH") {
        v.push(p);
    }

    // 2. Bundled — resolve relative to this .rlib's manifest dir
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let (os, arch) = platform();
    let bundled = manifest
        .parent().unwrap_or(manifest)
        .join("lib")
        .join(format!("{}-{}", os, arch))
        .join(lib_name());
    v.push(bundled.to_string_lossy().into_owned());

    // 3. Executable directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            v.push(dir.join(lib_name()).to_string_lossy().into_owned());
        }
    }

    // 4. System name (PATH/LD_LIBRARY_PATH)
    v.push(lib_name().to_string());
    v
}

fn platform() -> (&'static str, &'static str) {
    let os = match std::env::consts::OS {
        "windows" => "windows",
        "linux"   => "linux",
        "macos"   => "macos",
        other     => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64"  => "x64",
        "aarch64" => "arm64",
        other     => other,
    };
    (os, arch)
}

fn lib_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "overdrive.dll",
        "macos"   => "liboverdrive.dylib",
        _         => "liboverdrive.so",
    }
}

// ── Raw FFI symbol helpers ───────────────────────────────────────────────────

// BEFORE: `pub(crate) unsafe fn sym<T>(name: &[u8]) -> Symbol<'static, T>`
// called `.unwrap_or_else(|_| panic!(...))`. lib.rs didn't even use this
// helper — it called `lib.get(name).unwrap()` directly at ~20 call sites,
// so a single renamed/missing/version-skewed native symbol crashed the
// process instead of returning a normal error to the caller.
//
// AFTER: returns a Result. Every public API method now does
// `let func = ffi::try_sym::<...>(b"...")?;` and propagates the error.
pub(crate) unsafe fn try_sym<T>(name: &'static [u8]) -> Result<Symbol<'static, T>, SdkError> {
    let lib = try_load()?;
    lib.get(name).map_err(|e| {
        SdkError(format!(
            "[overdrive-sdk] symbol not found: {} ({})",
            String::from_utf8_lossy(name),
            e
        ))
    })
}

// BEFORE: `pub(crate) fn to_cstr(s: &str) -> CString` silently swallowed a
// `CString::new` failure (which happens whenever `s` contains an embedded
// NUL byte — e.g. a JSON document value with a `\0` inside a string field)
// and substituted an EMPTY string. Callers like `insert()` then silently
// wrote empty/garbage data to the database instead of erroring — a silent
// data-corruption bug that is worse than a panic because nothing is ever
// reported to the caller.
//
// AFTER: returns a Result; callers now get a real error instead of silent
// data loss.
pub(crate) fn try_to_cstr(s: &str) -> Result<CString, SdkError> {
    CString::new(s).map_err(|e| {
        SdkError(format!(
            "[overdrive-sdk] value contains an embedded NUL byte at offset {}; \
             refusing to silently truncate/corrupt the payload",
            e.nul_position()
        ))
    })
}

pub(crate) unsafe fn read_and_free(ptr: *mut c_char) -> Result<String, SdkError> {
    let lib = try_load()?;
    let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    let free: Symbol<unsafe extern "C" fn(*mut c_char)> = lib
        .get(b"overdrive_free_string")
        .map_err(|e| SdkError(format!("[overdrive-sdk] symbol not found: overdrive_free_string ({})", e)))?;
    free(ptr);
    Ok(s)
}

pub(crate) unsafe fn read_static(ptr: *const c_char) -> String {
    if ptr.is_null() { return String::new(); }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}
