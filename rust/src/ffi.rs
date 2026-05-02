use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use libloading::{Library, Symbol};
use std::sync::OnceLock;

static LIB: OnceLock<Library> = OnceLock::new();

/// Load the native overdrive library (cached after first call).
pub(crate) fn load() -> &'static Library {
    LIB.get_or_init(|| {
        // Resolution order:
        // 1. OVERDRIVE_LIB_PATH env var
        // 2. lib/{os}-{arch}/overdrive.dll (bundled)
        // 3. executable directory
        // 4. system PATH
        let candidates = lib_candidates();
        for path in &candidates {
            if let Ok(lib) = unsafe { Library::new(path) } {
                return lib;
            }
        }
        panic!(
            "[overdrive-sdk] Could not load native library.\n\
             Tried: {}\n\
             Set OVERDRIVE_LIB_PATH to point to overdrive.dll",
            candidates.join(", ")
        );
    })
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

pub(crate) unsafe fn sym<T>(name: &[u8]) -> Symbol<'static, T> {
    load().get(name).unwrap_or_else(|_| {
        panic!("[overdrive-sdk] symbol not found: {}", String::from_utf8_lossy(name))
    })
}

pub(crate) fn to_cstr(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("").unwrap())
}

pub(crate) unsafe fn read_and_free(ptr: *mut c_char) -> String {
    let lib = load();
    let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    let free: Symbol<unsafe extern "C" fn(*mut c_char)> =
        lib.get(b"overdrive_free_string").unwrap();
    free(ptr);
    s
}

pub(crate) unsafe fn read_static(ptr: *const c_char) -> String {
    if ptr.is_null() { return String::new(); }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}
