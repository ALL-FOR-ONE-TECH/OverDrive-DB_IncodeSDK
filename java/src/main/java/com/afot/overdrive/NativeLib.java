package com.afot.overdrive;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

/**
 * JNA interface binding to the native overdrive library.
 * Platform-arch resolution order:
 *   1. OVERDRIVE_LIB_PATH env var
 *   2. lib/{os}-{arch}/overdrive (bundled)
 *   3. System library path
 */
interface NativeLib extends Library {

    NativeLib INSTANCE = load();

    static NativeLib load() {
        // 1. Env override
        String envPath = System.getenv("OVERDRIVE_LIB_PATH");

        // 2. Bundled lib/{os}-{arch}/
        String libName = libName();
        String bundledPath = bundledPath(libName);

        String path;
        if (envPath != null && !envPath.isEmpty()) {
            path = envPath;
        } else if (bundledPath != null) {
            // Set JNA library path so it finds the dll
            System.setProperty("jna.library.path",
                    bundledPath.substring(0, bundledPath.lastIndexOf(java.io.File.separator)));
            path = libBaseName(libName);
        } else {
            path = libBaseName(libName);
        }

        return Native.load(path, NativeLib.class);
    }

    static String libName() {
        String os = System.getProperty("os.name", "").toLowerCase();
        if (os.contains("win"))    return "overdrive.dll";
        if (os.contains("mac"))   return "liboverdrive.dylib";
        return "liboverdrive.so";
    }

    static String libBaseName(String libName) {
        // JNA wants the base name without extension or "lib" prefix
        if (libName.startsWith("lib")) libName = libName.substring(3);
        int dot = libName.lastIndexOf('.');
        return dot > 0 ? libName.substring(0, dot) : libName;
    }

    static String platformArch() {
        String os   = System.getProperty("os.name", "").toLowerCase();
        String arch  = System.getProperty("os.arch", "").toLowerCase();
        String osKey = os.contains("win") ? "windows" : os.contains("mac") ? "macos" : "linux";
        String archKey = (arch.equals("amd64") || arch.equals("x86_64")) ? "x64"
                       : arch.equals("aarch64") ? "arm64" : arch;
        return osKey + "-" + archKey;
    }

    static String bundledPath(String libName) {
        // Walk up from this class's location to find lib/{os}-{arch}/
        java.net.URL url = NativeLib.class.getProtectionDomain().getCodeSource().getLocation();
        if (url == null) return null;
        java.io.File base;
        try { base = new java.io.File(url.toURI()); } catch (Exception e) { return null; }

        // Try up to 5 levels up
        java.io.File dir = base;
        for (int i = 0; i < 5; i++) {
            java.io.File candidate = new java.io.File(dir, "lib/" + platformArch() + "/" + libName);
            if (candidate.exists()) return candidate.getAbsolutePath();
            dir = dir.getParentFile();
            if (dir == null) break;
        }
        return null;
    }

    // ── Core ─────────────────────────────────────────────────────────────────
    Pointer overdrive_open(String path);
    Pointer overdrive_open_with_engine(String path, String engine, String optsJson);
    void    overdrive_close(Pointer handle);
    void    overdrive_sync(Pointer handle);
    String  overdrive_version();
    void    overdrive_free_string(Pointer ptr);
    String  overdrive_last_error();

    // ── Tables ────────────────────────────────────────────────────────────────
    int     overdrive_create_table(Pointer handle, String name);
    int     overdrive_drop_table(Pointer handle, String name);
    Pointer overdrive_list_tables(Pointer handle);
    int     overdrive_table_exists(Pointer handle, String name);

    // ── CRUD ──────────────────────────────────────────────────────────────────
    Pointer overdrive_insert(Pointer handle, String table, String json);
    Pointer overdrive_get(Pointer handle, String table, String id);
    int     overdrive_update(Pointer handle, String table, String id, String json);
    int     overdrive_delete(Pointer handle, String table, String id);
    int     overdrive_count(Pointer handle, String table);

    // ── Query ─────────────────────────────────────────────────────────────────
    Pointer overdrive_query(Pointer handle, String sql);
    Pointer overdrive_search(Pointer handle, String table, String text);

    // ── Transactions ─────────────────────────────────────────────────────────
    long    overdrive_begin_transaction(Pointer handle, int isolationLevel);
    int     overdrive_commit_transaction(Pointer handle, long txnId);
    int     overdrive_abort_transaction(Pointer handle, long txnId);

    // ── Integrity ────────────────────────────────────────────────────────────
    Pointer overdrive_verify_integrity(Pointer handle);
}
